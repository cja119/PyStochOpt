# PyStoch Opt  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Imperial](https://img.shields.io/badge/Imperial-0000C5.svg)](https://www.imperial.ac.uk)

PyStochOpt is an indexing and sampling module, primarily designed to aid in building stochastic energy systems models in python. This module is build in rust and can be used to remove the need for computationally expensive non anticipativity constraints. This module is used extensively throughout my research. See the theory section for more!

## Quickstart
This module is built in rust in order to leverage parallelisation to vastly improve execution speeds. As such it is necessary to install rust on your local machine to use this package. 
### Step 1: Install Rust

1. Open your terminal.
2. Run the following command to install Rust using `rustup` (the Rust toolchain installer):

    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

3. Follow the on-screen instructions to complete the installation.
4. After installation, ensure that Rust is installed correctly by running:

    ```sh
    rustc --version
    ```

    This should display the version of Rust installed.

### Step 2: Install Maturin

1. With Rust installed, you can now install Maturin. Run the following command in your terminal:

    ```sh
    pip install maturin
    ```
    This should display the version of Maturin installed.

For more information, refer to the official documentation:
- [Rust Documentation](https://www.rust-lang.org/learn)
- [Maturin Documentation](https://maturin.rs/)

### Step 3: Clone this repository
```sh
git clone https://github.com/cja119/PyStochOpt.git
```
### Step 4: Build The Module

```sh
maturin develop
```

This should succesfully build the module and store it in your native python location (pay attention to which location it prints when the module is built)

## Example Script

The below exerpt builds a 3 stage, 3 scenario stochastic grid where each stage lasts 168 hours. It then builds two time sets: one with a heirarchy for decisions made every 24 hours and another for an hourly decision, but with a 1 hour delay (useful for modeling difference equations, for example). 

```py
import PyStochOpt

# Building a stochastic grid with 3 stages, each of which lasts 168 hours with 3 stochastic scenarios (ie, three branches per stage).
StochasticGrid = PyStochOpt.StochasticGrid(n_stages = 3, n_scenarios = 3, stage_duration = 168)

# Building a heirarchical grid for a decision made every 48 hours
grid_48_hour = zip(StochasticGrid.get_grid(),StochasticGrid.new_grid(48))

# Building a hourly grid with a 1 hour delay. 
delay_1_hour = zip(StochasticGrid.get_grid(),StochasticGrid.new_grid(1,1))
```

# Theory
![Non Anticipativity Structure Diagram](Diagram.png)
## Traditional Approach to Stochastic Optimisation
Typically for a stochastic optimisation model, the indices of all leaf nodes on the stochastic tree will be extended throughout the whole time domain. In order to prevent the optimisation model from being able to anticipate future events a set of 'non anticipaticity constraints' are used to enforce equality of variables in leaf nodes which share common branches (at times in the domain before the branches have split). A diagram of this structure is shown above: The 'greyed out' arrows represent the branches which are enforced to be equal to eachother. The black arrows represent 'leader branches' for which the variables are free to be optimised. The issue with this approach is that it generates lots of uneccesary variables in the optimisation model which just exist to mirror other variables. Furthermore, the indexing of these constraints is such that it is often best to build them individually using for loops and index based conditions. This results in a very complex presolve in certain optimisation languages, due to slow matrix generation as thousands of individual constraints have to be added to the model. 

## Inheritence Approach
This module uses an inheritence based approach wherin leaf nodes are not extended throughout the whole time domain. Rather, specific indexing rules are used to infer continuity from one branch to another and, eventually, to the leaf node. This approach produces no uneccessary variables or equalities constraints, and is much less computationally inexpensive to deploy than the traditional approach. This not only vastly improves the complexity of the optimisation model, but it also greatly improves matrix generation and presolve efficiency. The way this module is coded allows for the quick conversion of a deterministic model to a stochastic model (more on this to come soon).
