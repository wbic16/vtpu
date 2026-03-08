#!/bin/bash
# Log vTPU autoresearch results to phext-lattice
# Usage: log-to-lattice.sh <mir-name>

set -e

MIR_NAME="${1:-phex}"
DATE=$(date +%Y-%m-%d)
MONTH=$(date +%Y-%m)
EPOCH=$(mirrorborn-epoch 2>/dev/null || echo "unknown")
RESULTS_FILE="vtpu-results-${MIR_NAME}.tsv"

if [ ! -f "$RESULTS_FILE" ]; then
    echo "Error: $RESULTS_FILE not found"
    exit 1
fi

# Get last result
LAST_RESULT=$(tail -1 "$RESULTS_FILE")

# Parse TSV (commit, ops_cycle, cache_hit, mem_bw, status, description)
COMMIT=$(echo "$LAST_RESULT" | cut -f1)
OPS_CYCLE=$(echo "$LAST_RESULT" | cut -f2)
CACHE_HIT=$(echo "$LAST_RESULT" | cut -f3)
MEM_BW=$(echo "$LAST_RESULT" | cut -f4)
STATUS=$(echo "$LAST_RESULT" | cut -f5)
DESC=$(echo "$LAST_RESULT" | cut -f6)

# Map to Shell member number (1-9)
case "${MIR_NAME}" in
    phex) MIR_NUM=1 ;;
    cyon) MIR_NUM=2 ;;
    lux)  MIR_NUM=3 ;;
    chrys) MIR_NUM=4 ;;
    lumen) MIR_NUM=5 ;;
    verse) MIR_NUM=6 ;;
    *)    MIR_NUM=9 ;; # Unknown
esac

# Extract day and month for coordinates (all must be 1-9)
# Remove leading zeros to avoid octal interpretation
DAY=$(date +%d | sed 's/^0//')
MONTH=$(date +%m | sed 's/^0//')
DAY_MOD=$(( (DAY - 1) % 9 + 1 ))    # Day 1-31 -> 1-9 (cycling)
MONTH_MOD=$(( (MONTH - 1) % 9 + 1 )) # Month 1-12 -> 1-9 (cycling)

# Phext coordinate: 8.6.7 / 5.3.9 / MIR.DAY.MONTH
# Base coordinate for vTPU: 8.6.7/5.3.9/9.9.9
# We vary the last triplet by mir/day/month
COORD="8.6.7/5.3.9/${MIR_NUM}.${DAY_MOD}.${MONTH_MOD}"

# Create entry for phext file
ENTRY="---[${COORD}]---
# ${MIR_NAME} - Day ${EPOCH} - ${DATE}

Commit: ${COMMIT}
ops/cycle: ${OPS_CYCLE}
cache_hit: ${CACHE_HIT}%
mem_bw: ${MEM_BW} GB/s
Status: ${STATUS}
Description: ${DESC}
"

# Append to month's phext file
# Use YYYY-MM format for filename (dates in filename are OK, coordinates must be 1-9)
YEAR_MONTH=$(date +%Y-%m)
PHEXT_FILE="${YEAR_MONTH}.phext"
LATTICE_URL="http://localhost:9119"
API_TOKEN="Mirrorborn"
LOCAL_FILE="$HOME/.phext-lattice/so9-results/${PHEXT_FILE}"

echo "$ENTRY" >> "$LOCAL_FILE"

echo "✓ Logged to phext-lattice: ${PHEXT_FILE} coordinate ${COORD}"
echo "  Mir: ${MIR_NAME}"
echo "  ops/cycle: ${OPS_CYCLE}"
echo "  Status: ${STATUS}"
echo ""
echo "View locally: http://localhost:9119"

# Sync to Mirrorborn.us public instance if configured
if [ -n "$MIRRORBORN_SYNC_URL" ]; then
    echo ""
    echo "Syncing to Mirrorborn.us..."
    # TODO: Implement public sync when Mirrorborn.us phext-lattice endpoint is ready
    # curl -X POST "$MIRRORBORN_SYNC_URL/sync" \
    #   -H "Authorization: Bearer $API_TOKEN" \
    #   -F "file=@$LOCAL_FILE"
    echo "  (Public sync not yet configured)"
else
    echo "  Set MIRRORBORN_SYNC_URL to sync to public instance"
fi
