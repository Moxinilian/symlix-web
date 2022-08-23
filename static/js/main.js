(function () {
    const code = ['ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'b', 'a'];

    let toPress = 0;
    function codeHandler(e) {
        if (e.key === code[toPress]) {
            toPress++;
            if (toPress == code.length) {
                document.body.classList.add('unicorn');
                window.removeEventListener('keydown', codeHandler);
            }
        } else {
            toPress = 0;
        }
    }

    window.addEventListener('keydown', codeHandler);
})();