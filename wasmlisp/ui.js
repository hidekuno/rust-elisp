/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
export function get_ace_text() {
    let editor = ace.edit("editor");
    return editor.getSession().getValue();
}
export function set_ace_text(s) {
    let editor = ace.edit("editor");
    editor.setValue(s, -1);
}
export function init_ace() {
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
