use eframe::egui;
use egui::{Align, Color32, FontId, Layout, Margin, RichText, TextEdit, TextStyle};

// ---- Цвета (тёмная тема + акцент ВТБ) ------------------------------------
const ACCENT: Color32 = Color32::from_rgb(0x00, 0x5f, 0xf9);
const GREEN: Color32 = Color32::from_rgb(0x4f, 0xd1, 0x8c);
const RED: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);
const GRAY: Color32 = Color32::from_rgb(0x9d, 0xa9, 0xba);
const BTN_BG: Color32 = Color32::from_rgb(0x2a, 0x32, 0x42);

// ---- Справочник продуктов --------------------------------------------------
#[derive(Clone, Copy)]
pub struct ProductDef {
    pub name: &'static str,
    pub note: &'static str,
    pub points: i32,
    pub manual: bool,
    pub min_points: i32,
    pub max_points: i32,
}

impl ProductDef {
    const fn fixed(name: &'static str, note: &'static str, points: i32) -> Self {
        Self {
            name,
            note,
            points,
            manual: false,
            min_points: points,
            max_points: points,
        }
    }

    const fn manual(name: &'static str, note: &'static str, min_points: i32, max_points: i32) -> Self {
        Self {
            name,
            note,
            points: min_points,
            manual: true,
            min_points,
            max_points,
        }
    }
}

pub const ALL_PRODUCTS: &[ProductDef] = &[
    ProductDef::fixed("СС от 250", "Потр. кредит со страховкой", 332),
    ProductDef::fixed("АВС", "Потр. кредит без страховки", 137),
    ProductDef::fixed("Изп/СЗП", "ИЗП / ЗП Лайт / СЗП", 273),
    ProductDef::fixed("Дк пос", "Продажа ДК / стикера", 50),
    ProductDef::manual("Кк", "Кредитная карта / Стикер", 60, 90),
    ProductDef::fixed("ПДС", "", 224),
    ProductDef::fixed("НС фонд шт", "Накопительный счет от 10 тыс.", 189),
    ProductDef::fixed("Пенсия шт", "Перевод пенсии в ВТБ", 150),
    ProductDef::fixed("Зп Лайт шт", "", 273),
    ProductDef::fixed("Вклад сумма(шт)", "", 137),
    ProductDef::fixed("КСП сумма(шт)", "", 98),
    ProductDef::fixed("Сом шт", "Страхование от мошенничества", 65),
    ProductDef::fixed("ВТБ+", "Подписка ВТБ Плюс", 45),
    ProductDef::fixed("Биометрия", "Снятие слепка биометрии", 50),
    ProductDef::fixed("СБ", "Семейный банк", 33),
    ProductDef::manual("Выдача ЗП ДК", "активация POS", 41, 62),
    ProductDef::fixed("Стратегия на пять", "", 222),
    ProductDef::fixed("Драйвер Гарантия", "", 287),
    ProductDef::fixed("Выдача ипотеки", "", 135),
    ProductDef::fixed("ОПИФ/ДУ", "", 194),
    ProductDef::fixed("ПУ Привилегия", "", 98),
    ProductDef::fixed("Автопополнение СБП", "", 137),
    ProductDef::fixed("Реферал ДК", "", 100),
    ProductDef::fixed("Автоплатеж ЖКУ/Связь", "", 43),
    ProductDef::fixed("ВТБ Мобайл", "", 32),
];

pub struct Product {
    pub def: ProductDef,
    pub count: u32,
    pub points: i32,
}

// ---- Приложение ------------------------------------------------------------
pub struct KpiApp {
    products: Vec<Product>,
    clients_served: String,
    npl_connected: String,
    search: String,
    monthly_total: i64,
    notice: String,
    arm_reset: bool,
}

impl KpiApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;
        ctx.set_visuals(egui::Visuals::dark());

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(16.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(16.0, egui::FontFamily::Proportional),
        );
        ctx.set_style(style);

        Self {
            products: ALL_PRODUCTS
                .iter()
                .map(|def| Product {
                    def: *def,
                    count: 0,
                    points: def.points,
                })
                .collect(),
            clients_served: "0/0".to_owned(),
            npl_connected: "0/0".to_owned(),
            search: String::new(),
            monthly_total: load_monthly(),
            notice: String::new(),
            arm_reset: false,
        }
    }

    /// Дневной итог: рассчитывается на лету при каждом изменении счётчиков.
    fn daily_total(&self) -> i64 {
        self.products
            .iter()
            .map(|p| p.count as i64 * p.points as i64)
            .sum()
    }

    fn save_day(&mut self) {
        let points = self.daily_total();
        if points == 0 {
            self.notice = "Нет набранных баллов за день — сохранение пропущено.".to_owned();
            return;
        }
        self.monthly_total += points;
        save_monthly(self.monthly_total);
        for p in self.products.iter_mut() {
            p.count = 0;
        }
        self.arm_reset = false;
        self.notice = format!("День сохранён (+{} б.). Дневные счётчики обнулены.", points);
    }

    fn build_report(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("Итог дня: {}\n", today_dd_mm()));
        out.push('\n');
        out.push_str(&format!(
            "Клиентов обслужено - {}\n",
            self.clients_served.trim()
        ));
        out.push_str(&format!("Подключено НПЛ - {}\n", self.npl_connected.trim()));
        out.push('\n');
        for p in &self.products {
            out.push_str(&format!("{} - {}\n", p.def.name, p.count));
        }
        out.push('\n');
        out.push_str(&format!("Прк - 0/{}\n", self.daily_total()));
        out
    }

    fn header_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("KPI DSA — баллы за день").size(18.0).strong());
        ui.add_space(6.0);

        ui.columns(2, |cols| {
            stat_card(&mut cols[0], "Итог дня", &format!("{} б.", self.daily_total()), GREEN);
            stat_card(&mut cols[1], "Общий итог (месяц)", &format!("{} б.", self.monthly_total), ACCENT);
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let save_w = ui.available_width() * 0.56;
            if ui
                .add_sized(
                    [save_w, 44.0],
                    egui::Button::new(RichText::new("Сохранить в общий итог").strong())
                        .fill(ACCENT),
                )
                .clicked()
            {
                self.save_day();
            }

            let reset_label = if self.arm_reset {
                "Подтвердить сброс?"
            } else {
                "Сброс месяца"
            };
            let reset_fill = if self.arm_reset { RED } else { BTN_BG };
            if ui
                .add_sized(
                    [ui.available_width(), 44.0],
                    egui::Button::new(RichText::new(reset_label).strong()).fill(reset_fill),
                )
                .clicked()
            {
                if self.arm_reset {
                    self.monthly_total = 0;
                    save_monthly(0);
                    self.arm_reset = false;
                    self.notice = "Общий итог сброшен.".to_owned();
                } else {
                    self.arm_reset = true;
                    self.notice = "Нажмите «Подтвердить сброс?» ещё раз".to_owned();
                }
            }
        });
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);

        ui.label(RichText::new("Клиентов обслужено").size(13.0).weak());
        ui.add(TextEdit::singleline(&mut self.clients_served).desired_width(f32::INFINITY));

        ui.label(RichText::new("Подключено НПЛ").size(13.0).weak());
        ui.add(TextEdit::singleline(&mut self.npl_connected).desired_width(f32::INFINITY));

        ui.add_space(4.0);
        ui.label(RichText::new("Поиск по продуктам").size(13.0).weak());
        ui.add(
            TextEdit::singleline(&mut self.search)
                .hint_text("Введите название…")
                .desired_width(f32::INFINITY),
        );

        ui.add_space(4.0);
        if ui
            .add_sized(
                [ui.available_width(), 46.0],
                egui::Button::new(
                    RichText::new("Скопировать отчёт для чата")
                        .size(16.0)
                        .strong(),
                )
                .fill(ACCENT),
            )
            .clicked()
        {
            self.arm_reset = false;
            let report = self.build_report();
            if copy_to_clipboard(&report) {
                self.notice = format!("✓ Отчёт скопирован ({})", today_dd_mm());
            } else {
                self.notice =
                    "Не удалось скопировать: нужен HTTPS (безопасное соединение).".to_owned();
            }
        }

        if !self.notice.is_empty() {
            ui.add_space(2.0);
            ui.label(RichText::new(&self.notice).color(GREEN).size(14.0));
        }
    }

    fn products_ui(&mut self, ui: &mut egui::Ui) {
        let query = self.search.trim().to_lowercase();
        let mut any = false;

        for p in self.products.iter_mut() {
            let hay = format!("{} {}", p.def.name, p.def.note).to_lowercase();
            if !query.is_empty() && !hay.contains(&query) {
                continue;
            }
            any = true;
            product_card(ui, p);
        }

        if !any {
            ui.add_space(24.0);
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new("Ничего не найдено").weak().size(15.0));
            });
        }
    }
}

impl eframe::App for KpiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let panel_fill = ctx.style().visuals.panel_fill;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(panel_fill)
                    .inner_margin(Margin::symmetric(12.0, 12.0)),
            )
            .show(ctx, |ui| {
                self.header_ui(ui);
                self.controls_ui(ui);
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| self.products_ui(ui));
            });
    }
}

fn product_card(ui: &mut egui::Ui, p: &mut Product) {
    p.points = p.points.clamp(p.def.min_points, p.def.max_points);

    egui::Frame::group(ui.style())
        .inner_margin(Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(p.def.name).size(15.0).strong());
                    if !p.def.note.is_empty() {
                        ui.label(RichText::new(p.def.note).size(12.0).weak());
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    ui.label(RichText::new(format!("{} б.", p.points)).size(13.0).color(GRAY));
                });
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .add_sized([44.0, 34.0], egui::Button::new(RichText::new("-").size(18.0)))
                    .clicked()
                {
                    p.count = p.count.saturating_sub(1);
                }
                ui.add_sized(
                    [40.0, 34.0],
                    egui::Label::new(RichText::new(p.count.to_string()).size(18.0).strong())
                        .selectable(false),
                );
                if ui
                    .add_sized([44.0, 34.0], egui::Button::new(RichText::new("+").size(18.0)))
                    .clicked()
                {
                    p.count += 1;
                }

                if p.def.manual {
                    ui.add_space(6.0);
                    ui.add(
                        egui::DragValue::new(&mut p.points)
                            .range(p.def.min_points..=p.def.max_points)
                            .speed(1.0)
                            .suffix(" б."),
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let row = p.count as i64 * p.points as i64;
                    if row > 0 {
                        ui.label(
                            RichText::new(format!("= {} б.", row))
                                .size(15.0)
                                .strong()
                                .color(GREEN),
                        );
                    }
                });
            });
        });
}

// ---- Вспомогательные виджеты ------------------------------------------------
fn stat_card(ui: &mut egui::Ui, title: &str, value: &str, color: Color32) {
    egui::Frame::group(ui.style())
        .inner_margin(Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).size(13.0).weak());
            ui.label(RichText::new(value).size(20.0).strong().color(color));
        });
}

// ---- Дата (DD.MM) -----------------------------------------------------------
#[cfg(target_arch = "wasm32")]
fn today_dd_mm() -> String {
    let d = js_sys::Date::new_0();
    format!("{:02}.{:02}", d.get_date(), d.get_month() + 1)
}

#[cfg(not(target_arch = "wasm32"))]
fn today_dd_mm() -> String {
    use chrono::Datelike;
    let now = chrono::Local::now();
    format!("{:02}.{:02}", now.day(), now.month())
}

// ---- Локальное хранилище: общий итог за месяц -------------------------------
#[cfg(target_arch = "wasm32")]
const KEY_MONTHLY_TOTAL: &str = "kpi_monthly_total";

#[cfg(target_arch = "wasm32")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
}

#[cfg(target_arch = "wasm32")]
fn load_monthly() -> i64 {
    storage()
        .and_then(|s| s.get_item(KEY_MONTHLY_TOTAL).ok())
        .flatten()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
fn save_monthly(value: i64) {
    if let Some(s) = storage() {
        let _ = s.set_item(KEY_MONTHLY_TOTAL, &value.to_string());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn state_file() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("kpi_monthly.txt")
}

#[cfg(not(target_arch = "wasm32"))]
fn load_monthly() -> i64 {
    std::fs::read_to_string(state_file())
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .unwrap_or(0)
}

#[cfg(not(target_arch = "wasm32"))]
fn save_monthly(value: i64) {
    let _ = std::fs::write(state_file(), value.to_string());
}

// ---- Буфер обмена ------------------------------------------------------------
#[cfg(target_arch = "wasm32")]
fn copy_to_clipboard(text: &str) -> bool {
    let Some(nav) = web_sys::window().map(|w| w.navigator()) else {
        return false;
    };
    let promise = nav.clipboard().write_text(text);
    wasm_bindgen_futures::spawn_local(async move {
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    });
    true
}

#[cfg(not(target_arch = "wasm32"))]
fn copy_to_clipboard(text: &str) -> bool {
    if let Ok(path) = std::env::current_dir() {
        let _ = std::fs::write(path.join("report_for_chat.txt"), text);
    }
    println!("--- Отчёт для чата ---\n{}\n------------------------", text);
    true
}
