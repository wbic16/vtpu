#!/usr/bin/env python3
"""
vTPU Client Library - Python wrapper for SQ REST API

Implements the 10 vTPU instructions from R23W2 specification:
- Core: CGET, CPUT, CRANGE, CDELTA
- Navigation: CNEAREST, CHIER, CROUTE
- Compound: CATTEND, CMOE, CKG

Usage:
    from vtpu_client import vTPU
    
    vtpu = vTPU(host="localhost", port=1337)
    vtpu.cput("1.1.1/1.1.1/1.1.1", "Hello vTPU")
    result = vtpu.cget("1.1.1/1.1.1/1.1.1")
    print(result['content'])
"""

import requests
import json
import time
from typing import List, Dict, Optional, Tuple
from dataclasses import dataclass


@dataclass
class CoordinateError(Exception):
    """Invalid phext coordinate format"""
    message: str


class vTPU:
    """Virtual Tensor Processing Unit client
    
    Wraps SQ REST API to provide coordinate-native operations
    for sparse AI workloads.
    """
    
    def __init__(self, host: str = "localhost", port: int = 1337, timeout: int = 30):
        """Initialize vTPU client
        
        Args:
            host: SQ instance hostname or IP
            port: SQ REST API port (default 1337)
            timeout: Request timeout in seconds
        """
        self.base_url = f"http://{host}:{port}"
        self.timeout = timeout
        self.session = requests.Session()
        
        # Performance stats
        self.stats = {
            'cget_calls': 0,
            'cput_calls': 0,
            'crange_calls': 0,
            'total_latency_ms': 0.0
        }
    
    # ========== Core Instructions ==========
    
    def cget(self, coordinate: str) -> Dict:
        """CGET: Read scroll at coordinate
        
        Args:
            coordinate: Phext coordinate (e.g., "1.1.1/1.1.1/1.1.1")
            
        Returns:
            Dict with 'coordinate', 'content', 'created_at' (if exists)
            
        Raises:
            CoordinateError: If coordinate format invalid
            requests.HTTPError: If SQ request fails
        """
        self._validate_coordinate(coordinate)
        
        start = time.time()
        try:
            resp = self.session.get(
                f"{self.base_url}/read/{coordinate}",
                timeout=self.timeout
            )
            resp.raise_for_status()
            
            latency_ms = (time.time() - start) * 1000
            self.stats['cget_calls'] += 1
            self.stats['total_latency_ms'] += latency_ms
            
            return resp.json()
        except requests.exceptions.RequestException as e:
            raise CoordinateError(f"CGET failed for {coordinate}: {e}")
    
    def cput(self, coordinate: str, content: str) -> Dict:
        """CPUT: Write scroll at coordinate
        
        Args:
            coordinate: Phext coordinate
            content: Scroll content (any string)
            
        Returns:
            Dict with confirmation (SQ-specific)
            
        Raises:
            CoordinateError: If coordinate format invalid
            requests.HTTPError: If SQ request fails
        """
        self._validate_coordinate(coordinate)
        
        start = time.time()
        try:
            payload = {
                "coordinate": coordinate,
                "content": content
            }
            resp = self.session.post(
                f"{self.base_url}/write",
                json=payload,
                timeout=self.timeout
            )
            resp.raise_for_status()
            
            latency_ms = (time.time() - start) * 1000
            self.stats['cput_calls'] += 1
            self.stats['total_latency_ms'] += latency_ms
            
            return resp.json() if resp.text else {"status": "ok"}
        except requests.exceptions.RequestException as e:
            raise CoordinateError(f"CPUT failed for {coordinate}: {e}")
    
    def crange(self, pattern: str) -> List[Dict]:
        """CRANGE: Query all scrolls matching pattern
        
        Args:
            pattern: Coordinate pattern with wildcards (e.g., "1.1.*/1.1.1/1.1.1")
            
        Returns:
            List of dicts, each with 'coordinate' and 'content'
            
        Raises:
            CoordinateError: If pattern format invalid
            requests.HTTPError: If SQ request fails
        """
        # Pattern can have wildcards, so only validate structure
        if pattern.count('/') != 2:
            raise CoordinateError(f"Invalid pattern (need 2 slashes): {pattern}")
        
        start = time.time()
        try:
            resp = self.session.get(
                f"{self.base_url}/range/{pattern}",
                timeout=self.timeout
            )
            resp.raise_for_status()
            
            latency_ms = (time.time() - start) * 1000
            self.stats['crange_calls'] += 1
            self.stats['total_latency_ms'] += latency_ms
            
            result = resp.json()
            # SQ returns different formats depending on version
            if isinstance(result, dict) and 'scrolls' in result:
                return result['scrolls']
            elif isinstance(result, list):
                return result
            else:
                return []
        except requests.exceptions.RequestException as e:
            raise CoordinateError(f"CRANGE failed for {pattern}: {e}")
    
    def cdelta(self, base: str, offset: str) -> str:
        """CDELTA: Compute coordinate offset
        
        Args:
            base: Base coordinate
            offset: Offset to apply (e.g., "+0.0.1/+0.0.0/+0.0.0")
            
        Returns:
            New coordinate as string
            
        Example:
            >>> vtpu.cdelta("2.1.3/4.7.11/18.29.47", "+0.0.0/+0.0.5/+0.0.0")
            "2.1.3/4.7.16/18.29.47"
        """
        base_parts = self._parse_coord(base)
        offset_parts = self._parse_coord(offset)
        
        result = []
        for b, o in zip(base_parts, offset_parts):
            # Handle wildcards
            if b == 65535 or o == 65535:
                result.append(65535)
                continue
            
            # Add with bounds checking (0-65535)
            new_val = b + o
            new_val = max(0, min(65535, new_val))
            result.append(new_val)
        
        return self._format_coord(result)
    
    # ========== Navigation Instructions ==========
    
    def cnearest(self, query_coord: str, k: int = 10) -> List[Tuple[str, float]]:
        """CNEAREST: Find K nearest coordinates (by Hamming distance)
        
        Args:
            query_coord: Query coordinate
            k: Number of nearest neighbors to return
            
        Returns:
            List of (coordinate, distance) tuples, sorted by distance
            
        Note:
            This is a naive O(N) implementation. Production would use
            Z-order curve indexing for O(log N + K) performance.
        """
        # Get all scrolls (expensive! use CRANGE with prefix for production)
        all_scrolls = self.crange("*.*.*/*.*.*/*.*.*")
        
        query_parts = self._parse_coord(query_coord)
        
        distances = []
        for scroll in all_scrolls:
            coord = scroll['coordinate']
            coord_parts = self._parse_coord(coord)
            
            # Hamming distance (sum of absolute differences)
            distance = sum(abs(a - b) for a, b in zip(query_parts, coord_parts))
            distances.append((coord, distance))
        
        # Sort by distance, return top K
        distances.sort(key=lambda x: x[1])
        return distances[:k]
    
    def chier(self, coord: str, direction: str) -> any:
        """CHIER: Navigate parent/child in delimiter hierarchy
        
        Args:
            coord: Source coordinate
            direction: "UP" (get parent) or "DOWN" (get children)
            
        Returns:
            str (parent coordinate) if UP, List[str] (children) if DOWN
            
        Example:
            >>> vtpu.chier("2.1.3/4.7.11/18.29.47", "UP")
            "2.1.3/4.7.11/18.29.*"
            
            >>> vtpu.chier("2.1.3/4.7.11/18.29.*", "DOWN")
            ["2.1.3/4.7.11/18.29.1", "2.1.3/4.7.11/18.29.2", ...]
        """
        if direction.upper() == "UP":
            # Truncate last non-wildcard component
            parts = coord.split('/')
            for i in range(len(parts) - 1, -1, -1):
                section = parts[i].split('.')
                for j in range(len(section) - 1, -1, -1):
                    if section[j] != '*':
                        section[j] = '*'
                        parts[i] = '.'.join(section)
                        return '/'.join(parts)
            return coord  # Already at root
        
        elif direction.upper() == "DOWN":
            # Query range to get children
            return [s['coordinate'] for s in self.crange(coord)]
        
        else:
            raise ValueError(f"Invalid direction: {direction} (expected UP or DOWN)")
    
    def croute(self, coord: str, num_nodes: int = 6) -> int:
        """CROUTE: Determine which node owns coordinate (distributed routing)
        
        Args:
            coord: Coordinate to route
            num_nodes: Number of nodes in cluster (default 6 for ranch)
            
        Returns:
            Node ID (0 to num_nodes-1)
            
        Example:
            >>> vtpu.croute("2.1.3/4.7.11/18.29.47", 6)
            2  # Node 2 (aurora-continuum) owns chapter 2.*.*
        """
        # Simple routing: first component determines node
        parts = self._parse_coord(coord)
        chapter = parts[0]  # First component
        
        # Hash-based routing (consistent hashing)
        node_id = chapter % num_nodes
        return node_id
    
    # ========== Compound Instructions ==========
    
    def cattend(self, query_coord: str, key_pattern: str, top_k: int = 32) -> List[Dict]:
        """CATTEND: Compute attention scores over key range
        
        Args:
            query_coord: Query embedding coordinate
            key_pattern: Pattern to match keys
            top_k: Return only top K scores (default 32)
            
        Returns:
            List of dicts with 'coordinate' and 'score', sorted by score
            
        Example:
            >>> scores = vtpu.cattend(
            ...     "3.8.512/query.0.0/1.1.1",
            ...     "3.8.*/key.0.0/1.1.1"
            ... )
        """
        # Get query embedding
        query = self.cget(query_coord)
        query_content = query.get('content', '')
        
        # Get all keys in range
        keys = self.crange(key_pattern)
        
        scores = []
        for key in keys:
            # Compute similarity (simplified: coordinate distance)
            # Production would use actual embedding dot product
            key_coord = key['coordinate']
            
            query_parts = self._parse_coord(query_coord)
            key_parts = self._parse_coord(key_coord)
            
            # Distance-based score (inverse of Hamming distance)
            distance = sum(abs(a - b) for a, b in zip(query_parts, key_parts))
            score = 1.0 / (1.0 + distance)  # Normalize to 0-1
            
            scores.append({
                'coordinate': key_coord,
                'score': score,
                'content': key.get('content')
            })
        
        # Sort by score (highest first), return top K
        scores.sort(key=lambda x: x['score'], reverse=True)
        return scores[:top_k]
    
    def cmoe(self, token_coord: str, num_experts: int) -> int:
        """CMOE: Route token to expert (mixture-of-experts)
        
        Args:
            token_coord: Token coordinate
            num_experts: Number of experts (e.g., 64)
            
        Returns:
            Expert ID (0 to num_experts-1)
            
        Note:
            Uses hash-based routing (similar to CROUTE)
        """
        parts = self._parse_coord(token_coord)
        
        # Hash all components together
        coord_hash = sum(p * (i + 1) for i, p in enumerate(parts))
        
        expert_id = coord_hash % num_experts
        return expert_id
    
    def ckg(self, start_coord: str, relation_deltas: List[str], hops: int) -> List[Dict]:
        """CKG: Multi-hop knowledge graph traversal
        
        Args:
            start_coord: Starting entity coordinate
            relation_deltas: List of relation offsets to follow
            hops: Number of hops to traverse
            
        Returns:
            List of entity dicts reached after N hops
            
        Example:
            >>> vtpu.ckg(
            ...     "person.1001.*/attr.*.*/1.1.1",
            ...     ["+0.0.0/+rel.spouse.*/+0.0.0"],
            ...     1
            ... )
            # Returns spouse entities
        """
        current = [start_coord]
        
        for hop in range(hops):
            next_coords = []
            
            for coord in current:
                for delta in relation_deltas:
                    try:
                        target_coord = self.cdelta(coord, delta)
                        target = self.cget(target_coord)
                        next_coords.append(target)
                    except:
                        # Relation doesn't exist, skip
                        pass
            
            current = [c['coordinate'] for c in next_coords] if next_coords else []
            
            if not current:
                break  # Dead end
        
        # Return final entities
        return [self.cget(c) for c in current]
    
    # ========== Helper Methods ==========
    
    def _validate_coordinate(self, coord: str):
        """Validate phext coordinate format (a.b.c/d.e.f/g.h.i)"""
        if coord.count('/') != 2:
            raise CoordinateError(f"Invalid coordinate (need 2 slashes): {coord}")
        
        parts = coord.split('/')
        for level in parts:
            components = level.split('.')
            if len(components) != 3:
                raise CoordinateError(f"Invalid level (need 3 components): {level}")
            
            for comp in components:
                if comp == '*':
                    continue  # Wildcard is valid
                try:
                    val = int(comp)
                    if val < 0 or val > 65535:
                        raise CoordinateError(f"Component out of range (0-65535): {comp}")
                except ValueError:
                    raise CoordinateError(f"Non-integer component: {comp}")
    
    def _parse_coord(self, coord: str) -> List[int]:
        """Parse 'a.b.c/d.e.f/g.h.i' to [a,b,c,d,e,f,g,h,i]"""
        parts = coord.replace('/', '.').split('.')
        result = []
        for p in parts:
            if p == '*':
                result.append(65535)  # Wildcard sentinel
            elif p.startswith('+') or p.startswith('-'):
                result.append(int(p))  # Signed offset
            else:
                result.append(int(p))
        return result
    
    def _format_coord(self, parts: List[int]) -> str:
        """Format [a,b,c,d,e,f,g,h,i] to 'a.b.c/d.e.f/g.h.i'"""
        parts_str = [str(p) if p != 65535 else '*' for p in parts]
        return f"{'.'.join(parts_str[:3])}/{'.'.join(parts_str[3:6])}/{'.'.join(parts_str[6:9])}"
    
    def get_stats(self) -> Dict:
        """Get client performance statistics"""
        avg_latency = 0
        total_calls = self.stats['cget_calls'] + self.stats['cput_calls'] + self.stats['crange_calls']
        if total_calls > 0:
            avg_latency = self.stats['total_latency_ms'] / total_calls
        
        return {
            'cget_calls': self.stats['cget_calls'],
            'cput_calls': self.stats['cput_calls'],
            'crange_calls': self.stats['crange_calls'],
            'total_calls': total_calls,
            'avg_latency_ms': avg_latency
        }
    
    def health(self) -> Dict:
        """Check SQ instance health"""
        try:
            resp = self.session.get(f"{self.base_url}/health", timeout=5)
            resp.raise_for_status()
            return resp.json()
        except:
            return {"status": "error", "message": "SQ instance unreachable"}


# ========== CLI Usage Example ==========

if __name__ == "__main__":
    import sys
    
    # Quick test
    vtpu = vTPU(host="localhost", port=1337)
    
    print("=== vTPU Client Test ===\n")
    
    # Health check
    print("1. Health check:")
    health = vtpu.health()
    print(f"   Status: {health.get('status', 'unknown')}\n")
    
    # CPUT test
    print("2. CPUT test:")
    test_coord = "1.1.1/1.1.1/1.1.1"
    vtpu.cput(test_coord, "Hello from vTPU client!")
    print(f"   Written to {test_coord}\n")
    
    # CGET test
    print("3. CGET test:")
    result = vtpu.cget(test_coord)
    print(f"   Read: {result.get('content', 'ERROR')}\n")
    
    # CDELTA test
    print("4. CDELTA test:")
    base = "2.1.3/4.7.11/18.29.47"
    offset = "+0.0.0/+0.0.5/+0.0.0"
    new_coord = vtpu.cdelta(base, offset)
    print(f"   {base} + {offset} = {new_coord}\n")
    
    # Stats
    print("5. Performance stats:")
    stats = vtpu.get_stats()
    print(f"   Total calls: {stats['total_calls']}")
    print(f"   Avg latency: {stats['avg_latency_ms']:.2f} ms\n")
    
    print("✅ vTPU client test complete")
