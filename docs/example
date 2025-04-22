# Example

Consider an energy systems model where you have a windfarm which powers a bank of electrolysers, which then feed into a set of gaseous hydrogen storage tanks. Each week you must decide how many trucks come and collect your hydrogen. Your plant is in a remote area and, as such, is 'islanded' (ie, doesn't have a grid connection). It is crucial that you do not run out of power, so you keep a hydrogen fuel cell on site which can 'cannibalise' some of your hydrogen from storage, in order to keep your safety systems available. You know a weeks' worth of weather in advance but know the weather for each subsequent week can be 'good', 'average' or 'poor'. 

The challenge here is knowing how many trucks to load with hydrogen each week in order to avoid running out of hydrogen incase of poor weather in the future (remember your safety systems can not be without power!). Let's model the system for a month (4 weeks operation), with our weather data avilable for 1 hours operation. For this, you can build the following stochstic system, and index as follows (example shown using Pyomo). 

1. __Importing the necessary packages__
```py
import PyStochOpt
from Pyomo.environ import Set, Param, ConcreteModel, Objective, minimize, NonNegativeReals, NonNegativeIntegers, Constraint
```
2. __Building the stochastic grid:__

Note that stage 0 (ie our root) always exists, so we have n_stages + 1 weeks duration. Building our base stochastic grid:
```py
stochastic_grid = PyStochOpt.StochasticGrid(n_stages = 3, n_scenarios = 3, stage_duration = 168)
```
3. __Loading the weatherdataset:__

Now generating samples from our weatherdataset. This needs to be a csv with 2 columns, one for time and one for the weatherdata value.
The sampler will pick a random point and then take a contiguous sample from it, building a full tree structure of samples.
```py
sampled_weather_data = stochastic_grid.add_dataset(file_name,file_path)
```
4.  __Building Heirarcichal Grids:__

Building a heirarchical grid for a decision made every 168 hours (one week) for our truck devisions. This will, for each time instance in each branch, 'point' to the correct index of the        168 hour grid set. Eg, at time = 175 on branch 1, this will point to the value at time = 168 on branch 1 as this was the last point at which a decision was made.
```py
grid_168_hour = zip(stochastic_grid.get_grid(),stochastic_grid.new_grid(168))
```
If our truck takes 8 hours to load with hydrogen, we may want to monitor the bays in our fuelling station. In order to do this we need to know how many
trucks started loading 8 hours ago, as they will now be finished. However, this can be challenging when trying to maintain contiguity for each trajectory.
For example at branch 2 at time = 172, we need to point into the past to time = 164. But, there is no branch 2 at time = 164 as we have crossed over a fork
in the stochstic tree, this is actually inherited from branch 0. Thankfully, PyStochOpt handles this complexity and returns the right value when you employ
the delay term. When we build our variable sets, we don't want a list of values which are repeated for each time point (as it will have lots of duplicates)
If we call the remove_duplicates functionality on this grid, it will allow us to generate a list containing single values. 

```py
grid_168_hour_no_duplicates = stochastic_grid.remove_duplicates(stochastic_grid.new_grid(168))
```

The need for this final grid will become more clear when we look at the hydrogen storage balance constriant.

```py
hydrogen_storage_grid = zip(stochastic_grid.get_grid(),stochastic_grid.new_grid(1,1),stochastic_grid.new_grid(168),stochastic_grid.new_grid(168,8))
````
7. __Building Repeat Stages__
A useful parameter to have is called "leaf nodes". This returns is the number of leaf nodes which correspond to a given branch node, which allows us to calculate correctly weighted averages (by repeat the number of branch nodes the correct number of times. This returns a dictionary, indexed in the same values as the stochastic grid.

```py
leaf_nodes = stochastic_grid.leaf_nodes()
```

6. __Putting it all together__
```py
# Let's build the model
model = ConcreteModel()

# Building Sets
model.time_tree = Set(initalize = stochastic_grid.get_grid(),dimen=2)
model.grid_168_hour = Set(initialize = grid_168_hour, dimen=4)
model.grid_168_hour_8_hour_delay = Set(grid_168_hour_8_hour_delay,dimen=6)
model.grid_168_hour_no_duplicates = Set(grid_168_hour_no_duplicates, dimen=2)
model.hydrogen_storage_grid = Set(hydrogen_storage_grid, dimen=8)

# Building Parameters
model.renewable_power = Param(model.time_tree, initialize=sampled_weather_data)
model.safety_systems_power = Param(initialise = 5) #GJ/H
model.leaf_nodes = Param(model.time_tree, leaf_nodesO

# Building Variables
model.stored_hydrogen = Var(model.time_tree, within=NonNegativeReals)
model.stored_hydrogen_capacity = Var(within=NonNegativeReals)
model.curtailed_energy = Var(model.time_tree, within=NonNegativeReals)
model.hydrogen_production = Var(model.time_tree, within=NonNegativeReals)
model.electrolyser_capacity = Var(within=NonNegativeReals)
model.number_trucks = Var(grid_168_hour_no_duplicates, within=NonNegativeIntegers)
model.hydrogen_cannibalistation = Var(model.time_tree, within=NonNegativeReals)


# Building Constraints
def energy_balance(self,s,t):
    return self.renewable_power[s,t] - self.curtailed_energy[s,t] + self.hydrogen_cannibalisation[s,t] * 120 * 0.5 - self.hydrogen_production[s,t] * 120 * 0.6 - self.safety_systems_power == 0

def hydrogen_storage_balance(self,s,t,s_0,t_0,s_t,t_t,s_t8,t_t8): 
    # s,t are from the normal decision tree.
    # s_0,t_0 are from the normal decision tree but with 1 hour delay
    # s_t,t_t are from the trucking decisions
    # s_t8, t_t8 are from the trucking devision but with an 8 hour delay (for loading).
    if t == 0:
        return Constraint.Skip
    if t_8 < t_t: # If our finish loading time is less than our start loading time. (When we are finished loading the pointers will point to the same value).
        return self.stored_hydrogen[s,t] - self.stored_hydrogen[s_0,t_0] - self.hydrogen_production[s,t] + self.number_trucks[s_t,t_t] * 2 == 0 # Assuming a charge rate of 2 t/h
    else:
        return self.stored_hydrogen[s,t] - self.stored_hydrogen[s_0,t_0] - self.hydrogen_production[s,t] == 0 

def hydrogen_storage_limit(self,s,t):
    reutrn self.stored_hydrogen[s,t] - self.stored_hydrogen_capacity <= 0

def hydrogen_production_limit(self,s,t):
    return self.hydrogen_production[s,t] <= self.electrolyser_capacity

def objective_function(self):
    # For simplicity's sake, this is just CAPEX + Discounted OPEX (at 4% of CAPEX) - Discounted Revenue (6% Discount factor 30 Yr lifetime)
    return  500000 * self.stored_hydrogen_capacity * (1.55) + 97200 * self.electrolyser_capacity * (1.55) \
            - 5000 * sum(self.leaf_nodes[s,t] * self.hydrogen_production[s,t] for s,t in self.time_tree) * (8760 / 672) * 13.67

model.hydrogen_production_limit = Constraint(model.time_tree, rule = hydrogen_storage_balance)
model.hydrogen_storage_balance = Constraint(model.hydrogen_storage_grid,rule=hydrogen_storage_balance)
model.energy_balance = Constraint(model.time_tree, rule = energy_balance)
model.objective = Objective(rule=objective_function, sense = minimise)

model.build_model()



```
