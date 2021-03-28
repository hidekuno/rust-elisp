#!/bin/sh
#
# "web/index.html" making tool
#
# ref.) https://blog.logrocket.com/getting-started-with-webassembly-and-rust/
#
# pre) wasm-pack build --target web --out-dir web
#
# hidekuno@gmail.com
#
FILE=web/index.html
SRC=./index.html

[ ! -d web ]  && exit 1

cat $SRC |sed '$d'|sed '$d'> $FILE
cat <<EOF>> $FILE
  <script type="module">
    import { default as wasm } from "./wasmlisp.js";
    wasm().then((module) => {console.log("Loading Success");});
  </script>
</html>
EOF

cp ui.js loading.gif web/.
