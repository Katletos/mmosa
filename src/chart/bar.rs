use plotters::prelude::*;

pub struct Bar<'a> {
    pub y_data: Vec<f64>,
    pub title: &'a str,
}

impl<'a> Bar<'a> {
    pub fn from_y_data(title: &'a str, data: Vec<f32>) -> Self {
        let y_data = data.into_iter().map(|v| v as f64).collect();

        Self { y_data, title }
    }

    pub fn save(&'a self, file_name: &str) -> std::io::Result<()> {
        let max_y = self
            .y_data
            .iter()
            .max_by(|a, b| a.total_cmp(b))
            .unwrap()
            .clone();

        let count = (self.y_data.len() * 3 + (self.y_data.len() - 1) * 3) as f64;
        let chart_name = format!("{file_name}.png");
        let root = BitMapBackend::new(&chart_name, (1024, 1024)).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption(self.title, ("Arial", 50).into_font())
            .margin(20)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0.0f64..count, 0.0f64..max_y)
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(self.y_data.iter().enumerate().map(|(index, v)| {
                let x0 = (index * 3 * 2) as f64;
                let x1 = x0 + 3.0;

                let y0 = *v;
                let y1 = 0.0;
                Rectangle::new([(x0, y0), (x1, y1)], BLUE.filled())
            }))
            .unwrap();

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();

        Ok(())
    }
}
