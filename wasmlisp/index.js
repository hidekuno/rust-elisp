/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
import('./pkg').catch(console.error);

(() => {

    let editor = ace.edit("editor");
    editor.$blockScrolling = Infinity;
    editor.setOptions({
        enableBasicAutocompletion: true,
        enableSnippets: true,
        enableLiveAutocompletion: true
    });
    editor.setTheme("ace/theme/textmate");
    editor.getSession().setMode("ace/mode/scheme");
    editor.setFontSize(12);

    let codearea = document.getElementById("codearea");
    let evalButton = document.getElementById("eval");

    evalButton.onmousedown = () => {
        codearea.value = editor.getSession().getValue();
    };

    evalButton.onkeydown = () => {
        if (event.keyCode == 32) {
            codearea.value = editor.getSession().getValue();
        }
    };

    document.getElementById("sicp").onclick = () => {
        editor.setValue('(load-url "z-learning/wasm-sicp.scm")', -1);
    };
    document.getElementById("demo").onclick = () => {
        editor.setValue('(demo)', -1);
    };
    document.getElementById("album").onclick = () => {
        editor.setValue('((below(beside rv ps)(beside sd am))(make-image-frame-rectangle "am" 2.2 2.2))', -1);
    };
})();
