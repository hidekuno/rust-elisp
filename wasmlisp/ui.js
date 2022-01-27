/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
const WEB_FONT = `<i class='fa fa-spinner fa-spin fa-5x fa-fw'></i><br><br>`;

const WAIT_MESSAGE = `Please wait until the alert dialog is displayed.`;

export function add_loading() {
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
export function set_textarea_from_ace() {
    let codeArea = document.getElementById("codearea");
    let editor = ace.edit("editor");
    codeArea.value = editor.getSession().getValue();
}
export function set_ace_text(s) {
    let editor = ace.edit("editor");
    editor.setValue(s, -1);
}
function init_ace() {
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
    })();
}
