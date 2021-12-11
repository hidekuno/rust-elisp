/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
const DEMO_CODE = `(draw-clear)
(define (draw-line-vect s e)(draw-line s e))
(demo)`;

const ANIMATION_DEMO_CODE = `(draw-clear)
(define (draw-line-vect s e)(add-timeout (draw-line s e) 10))
(demo)`;

const ALBUM_IMAGE_CODE = `(draw-clear)
((square-limit (below(beside rv ps)(beside sd am)) 0)
               (make-image-frame-rectangle "am" 1.74 1.74))`;

const WEB_FONT = "<i class='fa fa-spinner fa-spin fa-5x fa-fw'></i><br><br>";

const WAIT_MESSAGE = "Please wait until the alert dialog is displayed.";

function addLoading() {
    let ua = window.navigator.userAgent.toLowerCase();

    let msg  = (ua.indexOf('firefox') != -1)
        ?"<div class='loadingMsg'>" + WEB_FONT + WAIT_MESSAGE +"</div>"
        :"<div class='loadingMsg'>" + WAIT_MESSAGE +"</div>";

    if ($("#loading").length == 0) {
        $("body").append("<div id='loading'>" + msg + "</div>");
        if (ua.indexOf('firefox') == -1) {
            $('.loadingMsg').css('background', "url('/loading.gif') center center no-repeat");
        }
    }
}
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

    let codeArea = document.getElementById("codearea");
    let evalButton = document.getElementById("eval");

    evalButton.onmousedown = () => {
        addLoading();
        codeArea.value = editor.getSession().getValue();
    };
    document.getElementById("sicp").onclick = () => {
        editor.setValue('(load-url "wasm-sicp.scm")', -1);
    };
    document.getElementById("demo").onclick = () => {
        editor.setValue(DEMO_CODE, -1);
    };
    document.getElementById("anime").onclick = () => {
        editor.setValue(ANIMATION_DEMO_CODE, -1);
    };
    document.getElementById("album").onclick = () => {
        editor.setValue(ALBUM_IMAGE_CODE, -1);
    };
})();
