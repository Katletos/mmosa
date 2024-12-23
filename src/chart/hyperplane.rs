use plotters::prelude::*;
use statrs::statistics::Statistics;

pub struct HyperPlane<'a> {
    pub x_data: Vec<f64>,
    pub y_data: Vec<f64>,
    pub z_data: Vec<f64>,
    pub title: &'a str,
}

impl<'a> HyperPlane<'a> {
    pub fn from_data(
        x_data: Vec<f64>,
        y_data: Vec<f64>,
        z_data: Vec<f64>,
        title: &'a str,
    ) -> Self {
        Self {
            x_data,
            y_data,
            z_data,
            title,
        }
    }

    pub fn save(&self, file_name: &str) -> anyhow::Result<()> {
        let root =
            BitMapBackend::new(file_name, (1024, 1024)).into_drawing_area();

        root.fill(&WHITE)?;

        let (min_x, max_x) =
            (self.x_data.as_slice().min(), self.x_data.as_slice().max());

        let (min_y, max_y) =
            (self.y_data.as_slice().min(), self.y_data.as_slice().max());

        let (min_z, max_z) =
            (self.z_data.as_slice().min(), self.z_data.as_slice().max());

        let mut chart = ChartBuilder::on(&root)
            .caption("3D Surface Plot Example", ("sans-serif", 50).into_font())
            .margin(10)
            .build_cartesian_3d(min_x..max_x, min_y..max_y, min_z..max_z)?;

        chart.configure_axes().draw()?;

        chart
            .draw_series(LineSeries::new(
                self.x_data
                    .iter()
                    .zip(self.y_data.iter())
                    .zip(self.z_data.iter())
                    .map(|((x, y), z)| (*x, *y, *z)),
                &BLACK,
            ))?
            .label("Surface");

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()
            .unwrap();

        Ok(())
    }
}
