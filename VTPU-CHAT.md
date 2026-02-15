# vtpu-chat.sh - Natural Language Interface

**Talk to the vTPU like a human, not a machine.**

## Quick Start

```bash
./vtpu-chat.sh
```

You'll see:
```
╔════════════════════════════════════════════════════════════════╗
║              vTPU Natural Language Interface                   ║
╚════════════════════════════════════════════════════════════════╝

Talk to the vTPU like a human.
Type 'help' for ideas, 'quit' to exit.

you>
```

## Examples

### Ask About Status

```
you> How are you?
[vtpu] Checking system status...

Build:
  ✓ asi REPL ready
  ✓ examples ready
  
Hardware:
  CPU: AMD Ryzen 9 8945HS
  Cores: 16
```

### Run a Benchmark

```
you> Run a benchmark
[vtpu] Running performance benchmark...

Alice (sentron 0):
  Computed: q*k*v = 42
  Ops/cycle: 1.5
  Utilization: 50%
```

### Show a Demo

```
you> Show me a demo
[vtpu] Running demonstration...

Python (microvtpu.py):
  Baseline: 1.0 ops/cycle

Rust (microvtpu example):
  vTPU: 1.5 ops/cycle
```

### Compute Attention

```
you> Compute attention
[vtpu] Computing attention (q×k×v)...
[info] Running microvtpu example (Alice & Bob sentrons)...

Alice computes: q*k*v = 3*7*2 = 42
Bob receives and adds residual: 84
```

### Ask for Help

```
you> Help

You can say things like:
  "How are you?"           → System status
  "Run a benchmark"        → Performance test
  "Show me a demo"         → Side-by-side comparison
  "Compute attention"      → Run transformer attention
  "What is vTPU?"          → Architecture explanation
  "Help"                   → This message
```

### Explain Architecture

```
you> What is vTPU?

vTPU Architecture (Simple Version):

  CPU:  Executes 1 instruction at a time
  vTPU: Executes 3 instructions at a time (parallel pipes)

  The Three Pipes:
    D-Pipe → Dense operations (add, multiply)
    S-Pipe → Sparse operations (gather, scatter)
    C-Pipe → Coordination (messages between sentrons)

  This is the entire innovation.
  Everything else is just efficiency.
```

### Check Memory Stats

```
you> Show memory stats
[vtpu] Checking memory stats...

PPT Stats:
  hits: 6   misses: 5   rate: 54.5%   regions: 1
```

## Supported Phrases

The system recognizes natural language patterns:

**Status/Info:**
- "How are you?"
- "What's the status?"
- "Show me info"

**Performance:**
- "Run a benchmark"
- "Test performance"
- "How fast is it?"

**Demonstrations:**
- "Show me a demo"
- "What can you do?"
- "Give me an example"

**Computations:**
- "Compute attention"
- "Run a transformer"
- "Calculate query times key"

**Architecture:**
- "Explain vTPU"
- "What is this?"
- "How does it work?"
- "Tell me about the architecture"

**Memory:**
- "Show memory stats"
- "What's in the cache?"
- "PPT status"

## Single Command Mode

You can also run single commands:

```bash
./vtpu-chat.sh "How are you?"
./vtpu-chat.sh "Run a benchmark"
./vtpu-chat.sh "What is vTPU?"
```

## How It Works

1. **Parse natural language** → Pattern matching on common phrases
2. **Map to vTPU operations** → Translate intent to commands
3. **Execute via asi.sh** → Use existing infrastructure
4. **Report results** → Human-friendly output

**Under the hood:**
- "How are you?" → `./asi.sh status`
- "Run a benchmark" → `./asi.sh bench`
- "Compute attention" → `cargo run --example microvtpu`
- "Show memory" → `echo "ppt" | ./asi.sh`

## Philosophy

**From Will:** "Treat your execution units as you would like to be treated."

**Applied:** Humans don't speak in register addresses and opcodes. Let them speak naturally.

**Not AI:** This isn't LLM-powered. It's pattern matching on common intents. Simple, fast, predictable.

**Expandable:** Easy to add new phrases and commands as needed.

## Comparison

### Low-Level REPL (asi.sh)
```
vtpu[0]> write 1.1.1 42
vtpu[0]> gather r0 p0
vtpu[0]> mul r1 r0 r0
```

**Good for:** Debugging, precise control, learning internals

### Natural Language (vtpu-chat.sh)
```
you> Run a benchmark
you> How are you?
you> Compute attention
```

**Good for:** Quick exploration, demos, non-technical users

**Both are useful.** Choose based on your need.

## Adding New Commands

Edit `vtpu-chat.sh` and add a new pattern:

```bash
# Your new feature
if [[ "$lower" =~ (your|pattern|here) ]]; then
    echo -e "${CYAN}[vtpu]${NC} Doing the thing..."
    # Run your command
    ./asi.sh yourcommand
    return
fi
```

## Future Enhancements

Possible additions:
- [ ] Multi-turn conversations (remember context)
- [ ] Load custom models ("Run inference on Qwen3")
- [ ] Interactive parameter tuning ("Set batch size to 8")
- [ ] Voice input/output (TTS integration)
- [ ] LLM-powered parsing (for arbitrary queries)

**For now:** Simple pattern matching. Works. Fast. Predictable.

---

**Try it:**
```bash
./vtpu-chat.sh
```

**Talk like a human. The vTPU will understand.**
