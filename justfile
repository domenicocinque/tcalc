cli *ARGS:
    cargo run -p cli {{ ARGS }}

web:
    cd web && wasm-pack build --target web
    npx serve web/pkg
