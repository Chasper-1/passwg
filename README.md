🚀 PASSWG: Extreme-Performance SIMD Generator

English Description | Описание на русском

<a name="english-version"></a>

English Version

PASSWG is a low-level password generator written in Rust, specifically designed for maximum data throughput on x86_64 architectures using AVX2 instructions.

🎯 Why is it fast?

The program ignores standard slow string generation methods and works directly with CPU registers:

AVX2 / SIMD: Generates and maps 32 symbols per clock cycle using 256-bit vectors.

Lock-Free Parallelism: Thanks to the Rayon library, the workload is distributed across all available cores (P-cores and E-cores) without mutex bottlenecks.

Zero Modulo Bias: Implements a rejection sampling algorithm to ensure perfect mathematical entropy (~6.52 bits per symbol).

📊 Benchmarks (Intel i3-12100f)

On a budget 4-core CPU, PASSWG delivers:

Speed: ~485,000,000 passwords/sec (20 chars).

Throughput: ~10.2 GB/s (RAM/Bus bottleneck).

Entropy: ~130 bits for a 20-character password.

🛠 Usage

passwg [length/words] [count] [flags] [other options]

Example: passwg 20 1 -w. The flag -w automatically switches the logic from characters to words. You don't need to specify the number of words separately if they are already specified at the beginning.

Output formats include --json or --csv. For file output, use -o <path>. IMPORTANT: You must manually add the .json or .csv extension. Use -s for the built-in benchmark.

⚙️ Build

RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo build --release


Important Details

Maximum speed is only achieved in fast mode -f and when writing to /dev/null. No standard SSD can handle ~10 GB/s, and console output is a major bottleneck.

⚠️ Disclaimer

The generator was developed with AI assistance. I am not planning to curate or support this project. Fork it and play with it as you wish. The project is considered complete as it fulfills its main purpose. All future fixes and features are your responsibility.

<a name="russian-version"></a>

<details>
<summary>🇷🇺 <b>Нажми сюда, чтобы прочитать описание на русском</b></summary>

О проекте

PASSWG — это низкоуровневый генератор паролей на Rust, спроектированный для достижения максимально возможной пропускной способности данных на архитектуре x86_64.

🎯 Почему это быстро?

Программа игнорирует стандартные медленные методы генерации строк и работает напрямую с регистрами процессора:

AVX2 / SIMD: За один такт процессора генерируется и мапится сразу 32 символа.

Lock-Free Parallelism: Благодаря библиотеке Rayon нагрузка распределяется по всем ядрам (P-cores и E-cores) без задержек на блокировки.

Zero Modulo Bias: Использование алгоритма Rejection Sampling гарантирует идеальную математическую энтропию (~6.52 бит на символ).

📊 Бенчмарки (Intel i3-12100f)

На бюджетном 4-ядерном процессоре PASSWG выдает следующие показатели:

Скорость: ~485 000 000 паролей/сек (20 симв.)

Пропускная способность: ~10.2 ГБ/сек

Энтропия: ~130 бит для 20-символьного пароля.

🛠 Возможности

Три режима ChaCha: Выбор между ChaCha8, 12 или 20 раундами (-r).

Режим Fast (-f): Максимальная оптимизация под наборы символов [A-Za-z0-0_-].

Режим слов (-w): Генерация читаемых фраз.

Форматы: Plain text, JSON, CSV.

Clipboard: Прямая вставка в буфер обмена Wayland (-c).

Использование

passwg [количество символов/слов] [количество паролей] [флаги] [остальное для флагов]

Например: passwg 20 1 -w. Флаг -w автоматически переключает логику с символов на слова. Для вывода используйте --json или --csv. Для записи в файл используйте -o <путь>. ВАЖНО: Расширение файла (.json или .csv) нужно указывать вручную. Для запуска бенчмарка используйте флаг -s.

⚙️ Сборка

cargo build --release


<details>
<summary>Жёсткие оптимизации под ваше железо (рекомендуется)</summary>

RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo b -r


К слову, именно на таких настройках был достигнут результат в 2 секунды.

</details>

Важные детали

Максимальная скорость достигается исключительно в режиме fast mode -f. Если у вас СИЛЬНО медленнее, то либо железо не тянет, либо вы забыли этот флаг, либо скомпилировали без оптимизаций.

Также 10 ГБ/с достижимы только при записи в /dev/null.

<details>
<summary>Для тех, кому нужно объяснить, почему так.</summary>
Ни один обычный SSD не переварит такой поток данных. Не мучайте железо. Вывод в консоль тоже ОЧЕНЬ медленный сам по себе.
</details>

Дисклеймер

Генератор писался с помощью нейросетей, и я не хочу курировать этот проект. Делайте форк, играйтесь как хотите. Я просто поделился базой. Проект завершён, он работает. Все ошибки и доработки — теперь ваша забота. Не надейтесь на поддержку в основной ветке.

Дополнительная информация

По вопросам писать в Issues, по возможности отвечу.

Это не реклама ИИ, просто инструмент «для себя», которым решил поделиться.

Если код кажется кривым — сделайте лучше в своём форке. Это готовая база, пользуйтесь.

<img width="657" height="147" alt="benchmark_result" src="https://github.com/user-attachments/assets/9a6a9fe5-ae34-4eaf-8769-b49733d7d47c" />

</details>

License: MIT
