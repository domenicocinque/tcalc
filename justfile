cli *ARGS:
    cargo run -p cli {{ ARGS }}

web:
    cd web && wasm-pack build --target web --out-dir pkga
    npx serve web/pkg
