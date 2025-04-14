
## Building

Make sure pyenv is configured for version `3.10.X`.

```bash
python3 -m venv .env
source .env/bin/activate.fish
maturin develop --features="pyo3"
```

## Testing

In virtualenv: 
```bash
python3 -m pytest
```