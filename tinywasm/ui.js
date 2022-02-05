/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/

const PRIME_CODE =`(define (prime l)
  (if (> (car l)(sqrt (last l))) l
      (cons (car l)(prime (filter (lambda (n) (not (= 0 (modulo n (car l))))) (cdr l))))))

(prime (iota 30 2))`;

const PERM_CODE = `(define (perm l n)
  (if (>= 0 n) (list (list))
      (reduce (lambda (a b)(append a b))(list)
              (map (lambda (x) (map (lambda (p) (cons x p)) (perm (delete x l)(- n 1)))) l))))

(perm '(a b c) 2)`;

const COMB_CODE = `(define (comb l n)
  (if (null? l) l
      (if (= n 1) (map (lambda (n) (list n)) l)
          (append (map (lambda (p) (cons (car l) p)) (comb (cdr l)(- n 1)))
                  (comb (cdr l) n)))))

(comb '(a b c) 2)`;

const QSORT_CODE = `(define (qsort l pred)
  (if (null? l) l
      (append (qsort (filter (lambda (n) (pred n (car l))) (cdr l)) pred)
              (cons (car l) (qsort (filter (lambda (n) (not (pred n (car l))))(cdr l)) pred)))))

(define test-list (list 36 14 19 2 8 7 6 27 0 9 3))
(qsort test-list (lambda (a b)(< a b)))`;

const MSORT_CODE =`(define (l-merge a b)(if (or (null? a)(null? b)) (append a b)
  (if (< (car a)(car b))(cons (car a)(l-merge (cdr a) b))
         (cons (car b) (l-merge a (cdr b))))))
  (define (msort l)(let ((n (length l)))(if (>= 1 n ) l
    (if (= n 2) (if (< (car l)(cadr l)) l
     (reverse l))(let ((mid (quotient n 2)))(l-merge (msort (take l mid))(msort (drop l mid))))))))

(define test-list (list 36 14 19 2 8 7 6 27 0 9 3))
(msort test-list)
`
const BSORT_CODE =`(define (bubble-iter x l)
  (if (or (null? l)(< x (car l)))
      (cons x l)(cons (car l)(bubble-iter x (cdr l)))))

(define (bsort l)(if (null? l) l (bubble-iter (car l)(bsort (cdr l)))))

(define test-list (list 36 27 14 19 2 8 7 6 0 9 3))
(bsort test-list)
`

const WEB_FONT = `<i class='fa fa-spinner fa-spin fa-5x fa-fw'></i><br><br>`;

const WAIT_MESSAGE = `Please wait until the alert dialog is displayed.`;

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
class CodeHistory {
    constructor(select) {
        this.fifo = [];
        this.select = select;
    }
    push(code) {
        this.fifo.push(code);
        if (this.fifo.length > 10) {
            this.fifo.shift();
        }
        this.makeSelectOption();
    }
    makeSelectOption() {
        this.select.innerHTML = "";

        let option = document.createElement("option");
        option.text = "Please select code";
        option.value = -1;
        this.select.appendChild(option);

        let selectElment = this.select;
        this.fifo.forEach(function(value, idx) {
            let option = document.createElement("option");
            option.text = value.slice(0,80);
            option.value = idx;
            selectElment.appendChild(option);
        });
    }
    getValue(idx) {
        return this.fifo[idx];
    }
}
var codeHistory = new CodeHistory(document.querySelector('.history-code'));

(() => {
    const editor = ace.edit("editor");
    editor.$blockScrolling = Infinity;
    editor.setOptions({
        enableBasicAutocompletion: true,
        enableSnippets: true,
        enableLiveAutocompletion: true
    });
    editor.setTheme("ace/theme/textmate");
    editor.getSession().setMode("ace/mode/scheme");
    editor.setFontSize(12);

    const codeArea = document.getElementById("codearea");
    const evalButton = document.getElementById("eval");
    const selectElement = document.querySelector('.history-code');

    evalButton.onmousedown = () => {
        // addLoading();
        codeArea.value = editor.getSession().getValue();
    };
    selectElement.addEventListener(
        'change',
        (event) => {
            if (event.target.value != -1) {
                editor.setValue(codeHistory.getValue(event.target.value), -1);
            }
        });

    document.getElementById("prime").onclick = () => {
        editor.setValue(PRIME_CODE, -1);
    };
    document.getElementById("permutations").onclick = () => {
        editor.setValue(PERM_CODE, -1);
    };
    document.getElementById("combinations").onclick = () => {
        editor.setValue(COMB_CODE, -1);
    };
    document.getElementById("quicksort").onclick = () => {
        editor.setValue(QSORT_CODE, -1);
    };
    document.getElementById("mergesort").onclick = () => {
        editor.setValue(MSORT_CODE, -1);
    };
    document.getElementById("bubblesort").onclick = () => {
        editor.setValue(BSORT_CODE, -1);
    };
})();
