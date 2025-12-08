cli *ARGS:
    cargo run -p cli {{ ARGS }}

web:
    cd web && rm -rf pkg && wasm-pack build --target web --out-dir pkg && cp index.html pkg/
    cd web/pkg && npx serve


