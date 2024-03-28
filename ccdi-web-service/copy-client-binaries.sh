#!/bin/sh
set -e

CLIENT_DIR=../ccdi-web-client
CLIENT_DIST=$CLIENT_DIR/dist
DST_DIR=./src/static
sh -c "cd $CLIENT_DIR; trunk build --release"

WASM_FILE_SRC=$CLIENT_DIST/`ls -1 $CLIENT_DIST | grep ccdi-web-client | grep wasm`
JS_FILE_SRC=$CLIENT_DIST/`ls -1 $CLIENT_DIST | grep ccdi-web-client | grep js`
CSS_FILE_SRC=$CLIENT_DIST/`ls -1 $CLIENT_DIST | grep css`

WASM_FILE_DST=$DST_DIR/ccdi-web-client_bg.wasm
JS_FILE_DST=$DST_DIR/ccdi-web-client.js
CSS_FILE_DST=$DST_DIR/index.css

set +e
rm $WASM_FILE_DST
rm $JS_FILE_DST
rm $CSS_FILE_DST
set -e

echo "Copying $WASM_FILE_SRC -> $WASM_FILE_DST"
cp $WASM_FILE_SRC $WASM_FILE_DST
echo "Copying $JS_FILE_SRC -> $JS_FILE_DST"
cp $JS_FILE_SRC $JS_FILE_DST
echo "Copying $CSS_FILE_SRC -> $CSS_FILE_DST"
cp $CSS_FILE_SRC $CSS_FILE_DST