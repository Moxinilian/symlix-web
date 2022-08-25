(function () {
    Array.from(document.getElementsByClassName('toggler')).forEach((e, _) => {
        if (e.hasAttribute('data-toggling')) {
            let toggling = document.getElementById(e.dataset.toggling);
            e.addEventListener('click', () => {
                if (toggling.classList.contains('hidden')) {
                    toggling.classList.remove('hidden');
                    e.classList.add('open');
                } else {
                    toggling.classList.add('hidden');
                    e.classList.remove('open');
                }
            });
        } else {
            console.warn('toggler', e, 'does not have a toggling target');
        }
    });
})();
