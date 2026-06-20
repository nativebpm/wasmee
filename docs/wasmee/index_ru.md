# Семантический репозиторий Wasmee

Этот индекс содержит сведения о задачах и архитектурных изменениях движка durable WebAssembly Wasmee.

| ID | Название | Статус | Семантическое резюме |
| :--- | :--- | :--- | :--- |
| [WASMEE-101](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-101/WASMEE-101.md) | WASM Fiddle и интеграция с Git (прогрев) | Выполнено | Реализация загрузки из Git, распаковки ZIP-архивов, прогрева модулей и интерактивного интерфейса Fiddle. |
| [WASMEE-102](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-102/WASMEE-102.md) | Перевод рантайма в Stateless | Выполнено | Рефакторинг Wasmee с удалением глобального слоя персистентности RustStore для превращения в stateless-вычислитель. |
| [WASMEE-103](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-103/WASMEE-103.md) | GitOps без пайплайнов (Вебхуки и Кэш) | Выполнено | Прямая интеграция с Git через вебхуки, кэширование компиляций и ручную синхронизацию в UI для работы без CI/CD. |
| [WASMEE-104](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-104/WASMEE-104.md) | Раздел блога и документация производительности | Выполнено | Документирование компромисса между скоростью работы и сохранностью снимков состояния, а также создание раздела блога на сайте. |
| [WASMEE-105](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-105/WASMEE-105.md) | Облачное развертывание и верификация скрипта установки | Выполнено | Создание переносимого скрипта установки, сборка бинарных файлов для различных платформ и верификация установки в чистых Docker-контейнерах. |
| [WASMEE-106](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-106/WASMEE-106.md) | Запуск и верификация кода Fiddle через Git | Выполнено | Сборка гостевого модуля WebAssembly, отправка его в Git-репозиторий и проверка запуска с помощью локального демона wasmee и Fiddle UI. |
| [WASMEE-107](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-107/WASMEE-107.md) | Бизнес-выгоды и бенефиты интеграции в блоге | Выполнено | Добавление понятных коммерческих и интеграционных преимуществ durable WebAssembly в статью блога. |
| [WASMEE-108](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-108/WASMEE-108.md) | Интеграция WASM-модулей в стиле Go-модулей | Выполнено | Реализация надежного и нативного паттерна упаковки гостевых модулей WASM с использованием go:embed. |
| [WASMEE-109](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-109/WASMEE-109.md) | Мультиязычный SDK и гостевые примеры | В процессе | Создание отдельного публичного репозитория wasmee-sdk для хост-коннекторов и подготовка гостевых примеров на разных языках. |
