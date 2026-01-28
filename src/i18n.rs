#[allow(dead_code)]
pub struct I18n {
    pub help_usage: &'static str,
    pub help_args: &'static str,
    pub help_len: &'static str,
    pub help_count: &'static str,
    pub help_flags: &'static str,
    pub help_out: &'static str,
    pub help_json: &'static str,
    pub help_csv: &'static str,
    pub help_stats: &'static str,
    pub help_fast: &'static str,
    pub help_copy: &'static str,
    pub help_h: &'static str,
    pub stat_title: &'static str,
    pub stat_time: &'static str,
    pub stat_speed: &'static str,
    pub stat_perf: &'static str,
}

pub const EN: I18n = I18n {
    help_usage: "Usage: passwg [length] [count] [flags]",
    help_args: "Arguments:",
    help_len: "  length         Password length (default 16)",
    help_count: "  count          Number of passwords (default 1)",
    help_flags: "Flags:",
    help_out: "  -o <file>      Write output to file",
    help_json: "  --json         Output as JSON array",
    help_csv: "  --csv          Output as CSV",
    help_stats: "  -s, --stats    Show performance statistics",
    help_fast: "  -f, --fast     Max speed mode (A-Z, a-z, 0-9, _, -)",
    help_copy: "  -c, --copy     Copy one password to clipboard (Wayland only)",
    help_h: "  -h, --help     Show this help",
    stat_title: "STATISTICS",
    stat_time: "Execution time:   ",
    stat_speed: "Stream speed:     ",
    stat_perf: "Performance:      ",
};

pub const RU: I18n = I18n {
    help_usage: "Использование: passwg [длина] [количество] [флаги]",
    help_args: "Аргументы:",
    help_len: "  длина          Длина пароля (по умолчанию 16)",
    help_count: "  количество     Количество паролей (по умолчанию 1)",
    help_flags: "Флаги:",
    help_out: "  -o <file>      Записать вывод в файл",
    help_json: "  --json         Вывод в формате JSON массив",
    help_csv: "  --csv          Вывод в формате CSV",
    help_stats: "  -s, --stats    Показать статистику скорости",
    help_fast: "  -f, --fast     Режим макс. скорости (A-Z, a-z, 0-9, _, -)",
    help_copy: "  -c, --copy     Копировать один пароль в буфер (только Wayland)",
    help_h: "  -h, --help     Показать эту справку",
    stat_title: "СТАТИСТИКА",
    stat_time: "Время выполнения:  ",
    stat_speed: "Скорость потока:   ",
    stat_perf: "Производительность: ",
};

pub fn get_locale() -> &'static I18n {
    let lang = std::env::var("LANG").unwrap_or_default();
    if lang.starts_with("ru") { &RU } else { &EN }
}