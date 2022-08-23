(function () {
    Array.from(document.getElementsByClassName('toggler')).forEach((e, _) => {
        if (e.hasAttribute('data-toggling')) {
            let toggling = document.getElementById(e.dataset.toggling);
            let defaultDisplay =
                window.getComputedStyle(toggling).getPropertyValue('display');
            toggling.style.display = 'none';
            e.addEventListener('click', () => {
                if (toggling.style.display === 'none') {
                    toggling.style.display = defaultDisplay;
                    e.classList.add('open');
                } else {
                    toggling.style.display = 'none';
                    e.classList.remove('open');
                }
            });
        } else {
            console.warn('toggler', e, 'does not have a toggling target');
        }
    });
})();
