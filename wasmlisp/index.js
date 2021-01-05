/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
import('./pkg').catch(console.error);

const demo_code = `(draw-clear)
(define (draw-line-vect s e)
  (draw-line (xcor-vect s)(ycor-vect s)(xcor-vect e)(ycor-vect e)))
(demo)`;

const animation_demo_code = `(draw-clear)
(define (draw-line-vect s e)
  (add-timeout (draw-line (xcor-vect s)(ycor-vect s)(xcor-vect e)(ycor-vect e)) 10))
(demo)`;

const album_image_code = `(draw-clear)
((below(beside rv ps)(beside sd am))(make-image-frame-rectangle "am" 2.2 2.2))`;

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
        editor.setValue(demo_code, -1);
    };
    document.getElementById("anime").onclick = () => {
        editor.setValue(animation_demo_code, -1);
    };
    document.getElementById("album").onclick = () => {
        editor.setValue(album_image_code, -1);
    };
})();
