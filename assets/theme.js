(() => {
    // enhancement to load/store, inlined to avoid flash of wrong-theme
    const el = document.getElementById("theme");
    const meta = document.querySelector("meta[name='color-scheme']");
    const values = [...el.children].map(el => el.value);
    const storage_key = "dark-mode";

    function setTheme() {
        let value = el.value;
        window.localStorage.setItem(storage_key, value);
        if (value.includes("Light")) {
            meta.setAttribute("content", "light");
        } else if (value.includes("Dark")) {
            meta.setAttribute("content", "dark");
        } else {
            meta.setAttribute("content", "light dark");
        }
    }

    let stored = window.localStorage.getItem(storage_key);
    if (!stored || !values.includes(stored)) {
        stored = values[0];
    }
    el.value = stored;

    setTheme();

    el.addEventListener("change", setTheme);
})();