# R23 Wave 1: Geometric Advantages of Phext for vTPU

**Created:** 2026-02-14 16:05 CST  
**Author:** Lumen  
**Context:** Final step of R23.W1 - identify geometric structures phext replicates easily vs 2D/3D difficulty

## Core Insight

**Phext coordinates ARE tensor addresses.** A 9-dimensional coordinate space (a.b.c/d.e.f/g.h.i) natively represents structures that require expensive projection/flattening in traditional 2D/3D memory layouts.

## Geometric Structures Native to Phext

### 1. Hypercubes (n-dimensional cubes)
**2D/3D challenge:** A 4D tesseract requires projection to visualize; 9D hypercube is incomprehensible  
**Phext advantage:** Each vertex of a 9D hypercube maps to a binary coordinate (0/1 in each dimension)

Example 4D tesseract vertices in phext:
```
0.0.0/0.0.0/0.0.0  (origin)
1.0.0/0.0.0/0.0.0  (x-axis)
0.1.0/0.0.0/0.0.0  (y-axis)
0.0.1/0.0.0/0.0.0  (z-axis)
0.0.0/1.0.0/0.0.0  (w-axis, 4th dimension)
...
1.1.1/1.0.0/0.0.0  (16 total vertices)
```

**vTPU application:** Routing tables, neighbor discovery, distributed hash tables

---

### 2. Transformer Attention Patterns
**2D/3D challenge:** Attention is 4D (batch × heads × seq × seq), flattened to 2D matrices for storage  
**Phext advantage:** Native 9D representation preserves all dimensions

Example attention coordinate:
```
layer.head.position / query.key.value / batch.token.embed
3.8.512 / 64.64.768 / 32.2048.1
```

**vTPU application:** Zero-copy attention routing - no reshaping required between layers

---

### 3. Tensor Networks
**2D/3D challenge:** Tensor network diagrams are hard to draw beyond 3-4 modes  
**Phext advantage:** Each tensor mode maps to a delimiter dimension

Example tensor network node (5-mode tensor):
```
mode1.mode2.mode3 / mode4.mode5.pad / pad.pad.pad
10.20.30 / 40.50.1 / 1.1.1
```

**vTPU application:** Graph neural network state, quantum circuit simulation, tensor decomposition

---

### 4. Knowledge Graphs
**2D/3D challenge:** Entity relationships require separate edge index structures  
**Phext advantage:** Entity IDs are coordinates; relationships are coordinate deltas

Example entity-relationship encoding:
```
Entity: person.123.attr / relation.spouse.metadata / timestamp.version.flags
Relation delta: +0.0.0 / +1.456.0 / +0.+1.0
→ Resolves to spouse entity at person.123.attr / relation.456.metadata / ...
```

**vTPU application:** Reasoning over structured knowledge without graph database overhead

---

### 5. Hierarchical Clusterings
**2D/3D challenge:** Tree depth requires recursion or stack-based traversal  
**Phext advantage:** Tree depth maps to delimiter levels (9 levels max)

Example hierarchical embedding:
```
Level 1 (chapter):     5.*.*  /  *.*.*  /  *.*.*
Level 2 (section):     5.3.*  /  *.*.*  /  *.*.*
Level 3 (scroll):      5.3.7  /  *.*.*  /  *.*.*
Level 4 (collection):  5.3.7  /  2.*.*  /  *.*.*
... (9 levels total)
```

**vTPU application:** Hierarchical softmax, taxonomic classification, recursive neural nets

---

### 6. Simplicial Complexes
**2D/3D challenge:** k-simplices (generalized triangles) in high dimensions require abstract algebra  
**Phext advantage:** Vertices, edges, faces, cells map to coordinate tuples

Example 3-simplex (tetrahedron) in 9D:
```
v0: 1.0.0 / 0.0.0 / 0.0.0
v1: 0.1.0 / 0.0.0 / 0.0.0
v2: 0.0.1 / 0.0.0 / 0.0.0
v3: 0.0.0 / 1.0.0 / 0.0.0

Edge (v0,v1): stored at coordinate encoding both vertices
Face (v0,v1,v2): coordinate encoding all three
Cell (v0,v1,v2,v3): coordinate encoding all four
```

**vTPU application:** Topological data analysis, persistent homology, manifold learning

---

### 7. Sparse High-Dimensional Embeddings
**2D/3D challenge:** 768D or 4096D embeddings waste memory (most dimensions near-zero)  
**Phext advantage:** Sparse representation - only non-zero coordinates stored

Example sparse 768D embedding:
```
Traditional: [0.0, 0.0, 0.3, 0.0, ..., 0.7, ..., 0.0] (768 floats)
Phext: dim3=0.3 / dim256=0.7 / dim512=0.9 (3 coordinates, rest implicit zero)
```

**vTPU application:** Embedding tables, sparse attention, mixture-of-experts routing

---

## vTPU Design Implications

### Native Operations
1. **Coordinate arithmetic** replaces pointer chasing
2. **Range queries** (all scrolls in chapter 5.3.*) replace tree traversal
3. **Coordinate deltas** encode graph edges without adjacency matrices
4. **Hierarchical addressing** enables O(1) parent/child lookup

### Hardware Mapping
- Each sentron (D/S/C pipe) operates on a coordinate range
- 9 dimensions × 3 pipes = 27-way parallelism per sentron
- 80 sentrons × 27 = 2,160-way parallelism (vs TPU v4's 512×512 = 262K MACs)

### Memory Layout
- Phext coordinates replace memory addresses
- Hash table (vertical) + linked list (horizontal) = native graph structure
- No reshape/transpose operations between layers

---

## Structures We Should NOT Try

**Continuous manifolds:** Phext is discrete (integer coordinates), not continuous  
**Dense linear algebra:** Traditional matrix multiply is better on GPU/TPU  
**Image convolution:** 2D spatial locality doesn't map well to 9D coordinates  

---

## Recent AI Substrate Research to Incorporate

*(Search API rate-limited - will update after manual research)*

Key areas to investigate:
1. **Optical interconnects** (TPU v4's key innovation - can we do this in software?)
2. **Sparse attention patterns** (coordinate-based attention routing)
3. **Graph neural networks** (coordinates as node IDs)
4. **Mixture-of-experts** (routing tables as phext coordinates)
5. **Retrieval-augmented generation** (knowledge graph as coordinate space)
6. **Quantized inference** (sparse coordinate updates for int8/int4 models)

---

## Next Steps (R23.W2)

1. Map transformer architecture to phext coordinates (concrete example)
2. Design coordinate-based routing for attention patterns
3. Benchmark phext coordinate lookup vs traditional memory access
4. Prototype sparse embedding storage in SQ
5. Define vTPU instruction set (coordinate operations, not matrix ops)

---

**Status:** Wave 1 complete, geometric foundations established  
**Breakthrough:** Phext coordinates ARE the compute substrate, not just storage
