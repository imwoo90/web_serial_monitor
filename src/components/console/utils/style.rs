/// Returns (border_class, text_class) for a given highlight color name
pub fn get_highlight_classes(color: &str) -> (&'static str, &'static str) {
    match color {
        "red" => ("border-red-500/30 hover:border-red-500/60", "text-red-400"),
        "blue" => (
            "border-blue-500/30 hover:border-blue-500/60",
            "text-blue-400",
        ),
        "yellow" => (
            "border-yellow-500/30 hover:border-yellow-500/60",
            "text-yellow-400",
        ),
        "green" => (
            "border-green-500/30 hover:border-green-500/60",
            "text-green-400",
        ),
        "purple" => (
            "border-purple-500/30 hover:border-purple-500/60",
            "text-purple-400",
        ),
        "orange" => (
            "border-orange-500/30 hover:border-orange-500/60",
            "text-orange-400",
        ),
        "teal" => (
            "border-teal-500/30 hover:border-teal-500/60",
            "text-teal-400",
        ),
        "pink" => (
            "border-pink-500/30 hover:border-pink-500/60",
            "text-pink-400",
        ),
        "indigo" => (
            "border-indigo-500/30 hover:border-indigo-500/60",
            "text-indigo-400",
        ),
        "lime" => (
            "border-lime-500/30 hover:border-lime-500/60",
            "text-lime-400",
        ),
        "cyan" => (
            "border-cyan-500/30 hover:border-cyan-500/60",
            "text-cyan-400",
        ),
        "rose" => (
            "border-rose-500/30 hover:border-rose-500/60",
            "text-rose-400",
        ),
        "fuchsia" => (
            "border-fuchsia-500/30 hover:border-fuchsia-500/60",
            "text-fuchsia-400",
        ),
        "amber" => (
            "border-amber-500/30 hover:border-amber-500/60",
            "text-amber-400",
        ),
        "emerald" => (
            "border-emerald-500/30 hover:border-emerald-500/60",
            "text-emerald-400",
        ),
        "sky" => ("border-sky-500/30 hover:border-sky-500/60", "text-sky-400"),
        "violet" => (
            "border-violet-500/30 hover:border-violet-500/60",
            "text-violet-400",
        ),
        _ => ("border-primary/30 hover:border-primary/60", "text-primary"),
    }
}
