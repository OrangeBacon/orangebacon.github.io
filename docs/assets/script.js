// enhancement to load/store
function lightToggle(e) {
    let value = e.target.checked;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    value ^= media.matches;
    window.localStorage.setItem("dark-mode", Boolean(value).toString());
}

document.addEventListener("DOMContentLoaded", () => {
    const el = document.getElementById("light");

    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const stored = window.localStorage.getItem("dark-mode") === "true";
    el.checked = stored ^ media.matches;

    el.addEventListener("change", lightToggle);
});