use crate::{
    egui_charts::histogram::get_histogram, statistic::StatsConfig, EstimationConfig, Simulation,
    Stats,
};
use egui::Color32;
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotPoints, Points};

pub struct EguiApp {
    config: EstimationConfig,
    is_running: bool,
    data: Vec<f64>,
}

impl EguiApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = {
            let raw_config = std::fs::read_to_string("config.toml").expect("Failed to read config");
            toml::from_str::<EstimationConfig>(&raw_config).expect("Failed to parse config")
        };

        EguiApp {
            config,
            is_running: false,
            data: Vec::new(),
        }
    }
}

fn plot_histogram(ui: &mut egui::Ui, data: &[f64], name: &str) {
    let histogram = get_histogram(data);
    let bars: Vec<_> = histogram
        .bins
        .iter()
        .enumerate()
        .map(|(i, val)| {
            Bar::new(histogram.min + i as f64 * histogram.bin_width, *val)
                .width(histogram.bin_width)
                .fill(Color32::DARK_RED)
        })
        .collect();
    let chart = BarChart::new(bars.clone()).name(name);

    Plot::new(name)
        .view_aspect(2.0)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.add_item(Box::new(chart));
        });
    if data.len() > 1 {
        let stats = Stats::new_normal(data, data.len() - 1, &StatsConfig { alpha: 0.05 });
        // Stats{
        //     mean: todo!(),
        //     std_dev: todo!(),
        //     value_range: todo!(),
        //     t_stat: todo!(),
        //     chi_test: todo!(),
        //     ks_test: todo!(),
        //     bins: todo!(),
        // }
        ui.label(format!(
            "Mean: {:.4}\nStd_dev: {:.4}\nt_stat: {:.4}\nchi_test: {:?}\nks_test: {:?}",
            stats.mean, stats.std_dev, stats.t_stat, stats.chi_test, stats.ks_test
        ));
    }
}

fn plot_raw(ui: &mut egui::Ui, data: &[f64], name: &str) {
    if data.len() < 2 {
        return;
    }

    let plot_points: PlotPoints = data
        .iter()
        .enumerate()
        .map(|(i, val)| [i as f64, *val])
        .collect();

    let line = Points::new(plot_points).color(Color32::RED);
    let stats = Stats::new(data, &StatsConfig { alpha: 0.05 });
    let (start, end) = (stats.value_range.start, stats.value_range.end);

    let higher = Line::new(vec![[0.0, start], [data.len() as f64, start]]).color(Color32::GREEN);
    let lower = Line::new(vec![[0.0, end], [data.len() as f64, end]]).color(Color32::GREEN);

    Plot::new(name)
        .view_aspect(2.0)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.add(line);
            plot_ui.add(higher);
            plot_ui.add(lower);
        });
}

impl eframe::App for EguiApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Min).with_cross_justify(true),
                |ui| {
                    ui.vertical(|ui| {
                        let simulation_config = &mut self.config.simulation;
                        ui.heading("Simulation config");
                        if ui.button("Start/stop").clicked {
                            self.is_running = !self.is_running;
                        }
                        ui.add(
                            egui::Slider::new(&mut simulation_config.client_ratio, 0.0..=1.0)
                                .text("Client ratio"),
                        );
                        ui.add(
                            egui::Slider::new(&mut simulation_config.workers, 1..=100)
                                .text("Workers count"),
                        );
                        ui.add(
                            egui::Slider::new(&mut simulation_config.tables, 1..=100)
                                .text("Tables count"),
                        );
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(
                                &mut simulation_config.dancing_time.start,
                                1..=100,
                            ));
                            ui.add(
                                egui::Slider::new(&mut simulation_config.dancing_time.end, 1..=100)
                                    .text("Dancing time range"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(
                                &mut simulation_config.production_time.start,
                                1..=100,
                            ));
                            ui.add(
                                egui::Slider::new(
                                    &mut simulation_config.production_time.end,
                                    1..=100,
                                )
                                .text("Production time"),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add(egui::Slider::new(
                                &mut simulation_config.consumption_time.start,
                                1..=100,
                            ));
                            ui.add(
                                egui::Slider::new(
                                    &mut simulation_config.consumption_time.end,
                                    1..=100,
                                )
                                .text("Consumption time"),
                            );
                        });

                        if ui.button("Click to gen 100").clicked {
                            let mut sim = Simulation::with_config(self.config.simulation.clone());
                            for _ in 0..10_000 {
                                let (results, _) = sim.run();
                                self.data.push(results.average_free_workers as f64);
                                sim.reset_metrics();
                            }
                        }
                        if ui.button("reset simulation").clicked {
                            self.data.clear();
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("This is where logs are");
                        egui::containers::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.label("This is a scrollArea");

                                ui.horizontal(|ui| {
                                    ui.columns(2, |cols| {
                                        cols[0].vertical_centered_justified(|ui| {
                                            plot_histogram(ui, &self.data, "hi");
                                        });
                                        cols[1].vertical_centered_justified(|ui| {
                                            plot_raw(ui, &self.data, "biba");
                                        });
                                    });
                                });
                            });
                    });
                },
            );
        });

        if self.is_running {
            let mut sim = Simulation::with_config(self.config.simulation.clone());
            let (results, _) = sim.run();
            self.data.push(results.average_free_workers as f64);
            sim.reset_metrics();
        }
    }
}
