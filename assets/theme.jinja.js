(() => {
    // enhancement to load/store, inlined to avoid flash of wrong-theme
    const el = document.getElementById("theme");
    const values = [...el.children].map(el => el.value);
    const storage_key = "dark-mode";

    function setTheme(e) {
        let value = el.value;
        window.localStorage.setItem(storage_key, value);
    }

    let stored = window.localStorage.getItem(storage_key);
    if (!stored || !values.includes(stored)) {
        stored = values[0];
        window.localStorage.setItem(storage_key, stored);
    }
    el.value = stored;

    el.addEventListener("change", setTheme);
})();