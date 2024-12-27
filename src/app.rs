use crate::{
    egui_charts::histogram::get_histogram, statistic::StatsConfig, EstimationConfig, Log,
    Simulation, Stats,
};
use egui::Color32;
use egui_plot::{Bar, BarChart, Legend, Line, Plot, PlotBounds, PlotPoints, Points};

pub struct EguiApp {
    config: EstimationConfig,
    is_running: bool,
    data: Vec<f64>,
    free_workers_over_time: Vec<f64>,
}

impl EguiApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&_cc.egui_ctx);
        let config = {
            let raw_config = std::fs::read_to_string("config.toml").expect("Failed to read config");
            toml::from_str::<EstimationConfig>(&raw_config).expect("Failed to parse config")
        };

        EguiApp {
            config,
            is_running: false,
            data: Vec::new(),
            free_workers_over_time: Vec::new(),
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

    let points = Points::new(plot_points).color(Color32::RED);
    let stats = Stats::new(data, &StatsConfig { alpha: 0.05 });
    let (start, end) = (stats.value_range.start, stats.value_range.end);

    let higher = Line::new(vec![[0.0, start], [data.len() as f64, start]]).color(Color32::GREEN);
    let lower = Line::new(vec![[0.0, end], [data.len() as f64, end]]).color(Color32::GREEN);

    Plot::new(name)
        .view_aspect(2.0)
        .legend(Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.add(points);
            plot_ui.add(higher);
            plot_ui.add(lower);
        });
}

fn plot_line(ui: &mut egui::Ui, data: &[f64], name: &str) {
    if data.len() < 2 {
        return;
    }

    let plot_points: PlotPoints = data
        .iter()
        .enumerate()
        .map(|(i, val)| [i as f64, *val])
        .collect();

    let line = Line::new(plot_points).color(Color32::YELLOW);
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
            plot_ui.set_plot_bounds(PlotBounds::from_min_max([0.0, 0.0], [0.0, 20.0]));
            plot_ui.set_auto_bounds(egui::Vec2b::new(true, false));
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
                        ui.heading("Simulation config");
                        if ui.button("Start/stop").clicked {
                            self.is_running = !self.is_running;
                        }

                        if ui.button("Graph of free workers").clicked {
                            let mut sim = Simulation::with_config(self.config.simulation.clone());
                            let mut avg_log = Log::empty();
                            let amount_of_runs = 1000;

                            for _ in 0..amount_of_runs {
                                let (results, log) = sim.run();
                                sim.reset_metrics();
                                avg_log.add_mut(log);
                            }

                            avg_log.norm_mut(amount_of_runs);
                            self.free_workers_over_time = avg_log
                                .iter()
                                .map(|e| e.1.average_free_workers as f64)
                                .collect::<Vec<_>>();
                        }

                        let simulation_config = &mut self.config.simulation;
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

                        if ui.button("Click to gen 10").clicked {
                            let mut sim = Simulation::with_config(self.config.simulation.clone());
                            for _ in 0..10 {
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
                                ui.horizontal(|ui| {
                                    ui.columns(2, |cols| {
                                        cols[0].vertical_centered_justified(|ui| {
                                            plot_line(ui, &self.free_workers_over_time, "somedata");
                                            plot_histogram(ui, &self.data, "hi");
                                        });
                                        cols[1].vertical_centered_justified(|ui| {
                                            plot_raw(ui, &self.data, "biba");
                                        });
                                    });
                                });

                                // ui.horizontal(|ui| {
                                //     ui.columns(2, |cols| {
                                //         cols[0].vertical_centered_justified(|ui| {
                                //             plot_histogram(ui, &self.data, "hi");
                                //         });
                                //         cols[1].vertical_centered_justified(|ui| {
                                //             plot_raw(ui, &self.data, "biba");
                                //         });
                                //     });
                                // });
                                // ui.heading("3_1");
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_1/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.label(include_str!("../stats/3_1/BusyTables.toml"));
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_1/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.label(include_str!("../stats/3_1/DispatchedClients.toml"));
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_1/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.label(include_str!("../stats/3_1/FreeWorkers.toml"));
                                //
                                // ///////////////
                                //
                                // ui.heading("3_2");
                                //
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_2/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_2/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_2/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                //
                                // //////////
                                // ui.heading("3_3");
                                //
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_3/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_3/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_3/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // //////////
                                // ui.label("3_4");
                                //
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_4/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_4/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_4/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_4/NotDispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_4/WaitingTime.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // //////////
                                // ui.heading("3_5");
                                //
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_5/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_5/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_5/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_5/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_5/WaitingTime.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.label(include_str!("../stats/3_5/results.toml"));
                                //
                                // ui.heading("3_6");
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_6/WaitingTime.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_6/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_6/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/3_6/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.label(include_str!("../stats/3_6/results.toml"));
                                //
                                // ui.heading("4_1");
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_1/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_1/NotDispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_1/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_1/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_1/WaitingTime.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                //
                                // ui.heading("4_2: 1");
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_2/1/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_2/1/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_2/1/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_2/1/WaitingTime.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.label(include_str!("../stats/4_2/1/results.toml"));
                                //
                                // ui.heading("4_2: 2");
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_2/2/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_2/2/DispatchedClients.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_2/2/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/4_2/2/WaitingTime.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.label(include_str!("../stats/4_2/2/results.toml"));
                                //
                                // ui.heading("4_3");
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/multi/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/multi/WaitingTime.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                //
                                // ui.heading("logs");
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/logs/BusyTables.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/logs/FreeWorkers.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!(
                                //         "../stats/logs/WaitingTime.png",
                                //     ))
                                //     .fit_to_exact_size([1000.0, 1000.0].into())
                                //     .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!("../stats/logs/TS.png",))
                                //         .fit_to_exact_size([1000.0, 1000.0].into())
                                //         .rounding(5.0),
                                // );
                                // ui.add(
                                //     egui::Image::new(egui::include_image!("../stats/logs/TD.png",))
                                //         .fit_to_exact_size([1000.0, 1000.0].into())
                                //         .rounding(5.0),
                                // );
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
