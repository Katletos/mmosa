use ndarray_interp::interp2d::Interp2D;
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
        z_data: Vec<f64>,
        y_data: Vec<f64>,
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
        let (min_x, max_x) =
            (self.x_data.as_slice().min(), self.x_data.as_slice().max());

        let (min_y, max_y) =
            (self.y_data.as_slice().min(), self.y_data.as_slice().max());

        let (min_z, max_z) =
            (self.z_data.as_slice().min(), self.z_data.as_slice().max());

        let chart_name = format!("{file_name}.png");

        let root =
            BitMapBackend::new(&chart_name, (1024, 1024)).into_drawing_area();

        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(self.title, ("sans-serif", 50).into_font())
            .margin(10)
            .build_cartesian_3d(min_x..max_x, min_y..max_y, min_z..max_z)?;

        chart.configure_axes().draw()?;

        let x = ndarray::Array1::from_vec(self.x_data.clone());
        let z = ndarray::Array1::from_vec(self.z_data.clone());
        let y = ndarray::Array2::from_shape_vec(
            (x.len(), z.len()),
            self.y_data.clone(),
        )
        .unwrap();

        let x_iter = (((min_x * 100.0) as usize)..((max_x * 100.0) as usize))
            .map(|v| v as f64 / 100.0);
        let z_iter = (((min_z * 100.0) as usize)..((max_z * 100.0) as usize))
            .map(|v| v as f64 / 100.0);

        let interpolator = Interp2D::builder(y).x(x).y(z).build().unwrap();

        chart
            .draw_series(
                SurfaceSeries::xoz(x_iter, z_iter, |x, z| {
                    interpolator.interp_scalar(x, z).unwrap()
                })
                .style(&BLUE.mix(0.5)),
            )?
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
