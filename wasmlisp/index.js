import('./pkg').catch(console.error);

var editor = ace.edit("editor");
editor.$blockScrolling = Infinity;
editor.setOptions({
  enableBasicAutocompletion: true,
  enableSnippets: true,
  enableLiveAutocompletion: true
});

editor.setTheme("ace/theme/textmate");
editor.getSession().setMode("ace/mode/scheme");
editor.setFontSize(12)

var codearea = document.getElementById("codearea");
var eval_button = document.getElementById("eval");

eval_button.onmousedown = function() {
  codearea.value = editor.getSession().getValue();
}

eval_button.onkeydown = function(e) {
  if (event.keyCode == 32) {
    codearea.value = editor.getSession().getValue();
  }
}
