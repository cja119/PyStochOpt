use pyo3::prelude::*;
use rayon::prelude::*;
use pyo3::types::PyTuple;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyclass]
struct StochasticGrid {
    n_stages: usize,
    n_scenarios: usize,
    stage_duration: usize,
    grid: Vec<(usize,usize)> ,
}

fn build_grid(n_stages: usize, n_scenarios: usize,stage_duration: usize) ->  Vec<(usize,usize)> {
    let total_time: usize = stage_duration * (n_scenarios.pow(n_stages as u32) - 1) / (n_scenarios - 1) + ((n_stages) * stage_duration  - n_stages * stage_duration) +  n_scenarios.pow(n_stages as u32) * stage_duration;
    let mut grid: Vec<(usize,usize)>   = vec![(0,0); total_time];

    (0..((n_stages+1) * stage_duration)).into_iter().for_each(|t| {
        let stage: usize = (t as f64 / stage_duration as f64).floor() as usize;
        let scenario: usize = n_scenarios.pow(stage as u32);
        (0..scenario).into_iter().for_each(|s| {
            let key: usize = stage_duration * (n_scenarios.pow(stage as u32) - 1) / (n_scenarios - 1) + (t - stage * stage_duration) + s * stage_duration;
            grid[key] = (s, t);
        });
    });
    return grid;
}   

#[pymethods]
impl StochasticGrid {
    #[new]
    fn new(n_stages: usize, n_scenarios: usize, stage_duration:usize) -> Self {
        let grid:  Vec<(usize,usize)> =  build_grid(n_stages, n_scenarios, stage_duration);
        return StochasticGrid {n_stages, n_scenarios, stage_duration, grid};
    }

    fn get_grid(&self, py: Python<'_>) -> PyResult<Vec<PyObject>>  {
        
        let python_list: Vec<PyObject> = self.grid
            .clone()
            .into_iter()
            .map(|(x, y)| PyTuple::new_bound(py, &[x, y]).to_object(py))
            .collect();
        return  Ok(python_list);
    }

    #[pyo3(signature = (grid_duration, delay=None))]
    fn new_grid(&mut self, grid_duration: usize, delay: Option<usize>) -> Vec<(usize,usize)> {
        let total_time: usize = self.stage_duration * (self.n_scenarios.pow(self.n_stages as u32) - 1) / (self.n_scenarios - 1) + ((self.n_stages) * self.stage_duration  - self.n_stages * self.stage_duration) +  self.n_scenarios.pow(self.n_stages as u32) * self.stage_duration;
        let delay: usize = delay.unwrap_or(0);
        let mut new_grid: Vec<(usize,usize)>   = vec![(0,0); total_time];
    
        (0..((self.n_stages+1) * self.stage_duration)).into_iter().for_each(|t| {
            if t < delay {     
                let grid_t:usize = ((t as f64 / grid_duration as f64).floor() as usize) * grid_duration as usize;
                let grid_stage: usize = ((grid_t as f64 / self.stage_duration as f64).floor() as usize);
                let stage: usize = (t as f64 / self.stage_duration as f64).floor() as usize;
                let scenario: usize = self.n_scenarios.pow(stage as u32);      
                (0..scenario).into_iter().for_each(|s| {
                    let key: usize = self.stage_duration * (self.n_scenarios.pow(stage as u32) - 1) / (self.n_scenarios - 1) + (t - stage * self.stage_duration) + s * self.stage_duration;
                    let ratio: usize = self.n_stages.pow((stage - grid_stage) as u32);
                    new_grid[key] = ((s as f64/ratio as f64).floor() as usize, 0);
                }); 
            }
            else {
                let grid_t:usize = (((t - delay) as f64 / grid_duration as f64).floor() as usize) * grid_duration as usize;
                let grid_stage: usize = ((grid_t as f64 / self.stage_duration as f64).floor() as usize);
                let stage: usize = ((t - delay) as f64 / self.stage_duration as f64).floor() as usize;
                let scenario: usize = self.n_scenarios.pow(stage as u32);

                (0..scenario).into_iter().for_each(|s| {
                    let key: usize = self.stage_duration * (self.n_scenarios.pow(stage as u32) - 1) / (self.n_scenarios - 1) + (t - stage * self.stage_duration) + s * self.stage_duration;
                    let ratio: usize = self.n_stages.pow((stage - grid_stage) as u32);
                    new_grid[key] = ((s as f64/ratio as f64).floor() as usize, grid_t);
                });
        }
        });
        return new_grid;
    }
    

}


/// A Python module implemented in Rust.
#[pymodule]
fn PyStochOpt(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<StochasticGrid>()?;
    Ok(())
}