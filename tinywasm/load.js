/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
import { default as wasm } from "./pkg/tinywasm.js";
wasm().then((module) => {console.log("Loading Success");});
import { do_scheme } from "./pkg/tinywasm.js";

const evalButton = document.getElementById("eval");
const codeArea = document.getElementById("codearea");

evalButton.onclick = () => {
    codeHistory.push(codeArea.value);
    let v = do_scheme(codeArea.value);
    alert(v);
};
