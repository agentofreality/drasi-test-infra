The Building Comfort Test example demonstrates how to run a test using a model-based source ([Building Hierarchy Model Data Generator](../../test-run-host/src/sources/model_data_generators/building_hierarchy/mod.rs)). This will allow you to generate a large number of synthetic Source Change Events and dispatch them at configurable rates to a Drasi environment.

## Test Overview
The Building Comfort Test example uses a model-based source to generate a large number of synthetic Source Change Events and dispatch them at configurable rates. The test uses the following configuration:

| Setting          | Description                                                                 |
|------------------|-----------------------------------------------------------------------------|
| Source           | Building Hierarchy Source                                                  |
| Data             | 10 Buildings with 10 Floors with 10 Rooms for a total of 1000 Rooms. Each Room has 3 Sensors: temperature, humidity, and cO2. |
| Changes          | 100,000 Source Change Events, which result in ~96,000 observed Query Result Changes. |
| Rate | Changes are dispatched as quickly as possible.
| Continuous Query | ```MATCH (r:Room) RETURN elementId(r) AS RoomId, 50+(r.temperature-72)+(r.humidity-42)+(r.co2/25) AS ComfortLevel``` |

## Test Execution
There are two ways to run the Building Comfort example:
1. [On a Drasi environment](#executing-the-building-comfort-example-on-a-drasi-environment)
1. [In a local process](#executing-the-building-comfort-example-in-a-local-process)

## Executing the Building Comfort Example on a Drasi environment
The files for executing the Building Comfort Test example on a Drasi environment are located in the `test-infra/e2e-test-framework/examples/building_comfort/drasi` folder. 

These files assume:
1. you have the [Drasi CLI](https://drasi.io/reference/command-line-interface/) installed on the machine where you will launh the test from.
1. you have `kubectl` installed on the machine where you will launch the test from.
1. you have a Kubernetes cluster on which you want to run the test (e.g., Kind, AKS, etc.).
1. you have credentials to access the Drasi Test Repo containing the Building Comfort Test.

The folder contains the following files to launch, inspect, control, and end the test:

| File Name | Description |
|-----------|-------------|
| run_test_with_redis.sh | A shell script that installs Drasi and the E2E Test Framework on the current Kubernetes cluster and runs the Building Comfort test using the default Redis index. |
| run_test_with_redis_observability.sh | A shell script that installs Drasi and the E2E Test Framework on the current Kubernetes cluster, updates the installed observability stack, and runs the Building Comfort test using the default Redis index. |
| run_test_with_rocks.sh | A shell script that installs Drasi and the E2E Test Framework on the current Kubernetes cluster, updates the default Drasi Query Container to use RocksDB for its index storage, and runs the Building Comfort test. |
| run_test_with_memory.sh | A shell script that installs Drasi and the E2E Test Framework on the current Kubernetes cluster, updates the default Drasi Query Container to use an InMemory index, and runs the Building Comfort test. |
| end_test_run.sh | A shell script that stops the test and cleans up the Kubernetes cluster used during the test. |
| get_test_run_results.sh | A shell script that retrieves the test results from the test run and stores them in a local folder. |
| query_container_redis.yaml | A Drasi resource definition that configures the default Query Container to use Redis for its index storage. |
| query_container_rocks.yaml | A Drasi resource definition that configures the default Query Container to use RocksDB for its index storage. Used by the run_test_with_rocks.sh script. |
| query_container_memory.yaml | A Drasi resource definition that configures the default Query Container to use an InMemory index. Used by the run_test_with_memory.sh script. |
| test_service_deployment.yaml | A Kubernetes resource definition that deploys the E2E Test Service to the Kubernetes cluster and namespace where Drasi is installed. |
| source.yaml | A Drasi resource definition for the Source being tested. |
| query.yaml | A Drasi resource definition for the Continuous Query being tested. |
| web_api_source.http | A file used by the REST Client extension for Visual Studio Code to make it easy to interact with the E2E Test Service to inspect and control the Test Run Source. |
| web_api_query.http | A file used by the REST Client extension for Visual Studio Code to make it easy to interact with the E2E Test Service to inspect and control the Test Run Query. |

### Running the Test
To run the test, do the following:

1. Edit the `test_service_deployment.yaml` file and set the `access_key` proeprty of the the Test Repo named `az_dev_repo`. This is required for the Test Service to access the Drasi Test Repo containing the Building Comfort Test.
1. Open a terminal and navigate to the `test-infra/e2e-test-framework` folder.
1. Set the Kubernetes context of `kubectl` to the cluster on which you want to run the test. 
1. Run one of the `run_test_*.sh` scripts, depending on which index you want the Drasi Query Container to use during the test. For example, to run the test using RocksDB index storage, run the following command:

```bash
./examples/building_comfort/run_test_with_rocks.sh
```

Each of the `run_test_*.sh` scripts will do the following:
1. Install Drasi on the current Kubernetes cluster.
1. Optionally update the observability stack if the `run_test_with_redis_observability.sh` script is used.
1. Optionally update the Query Container to use RocksDB or InMemory index storage if the `run_test_with_rocks.sh` or `run_test_with_memory.sh` scripts are used.
1. Install the E2E Test Framework on the current Kubernetes cluster.
1. Install the Drasi Test Source Provider.
1. Create the Drasi Source used in the test.
1. Create the Drasi Continuous Query used in the test.
1. Forward the port of the E2E Test Service so you can use the Test Service Web API.
1. Start the test.

### Inspecting and Controlling the Test
You can always inspect and control the Test Run through the Test Service Web API, which will be accessible through `http://localhost:63123`. 

The `web_api_source.http` and `web_api_query.http` files in the `drasi` folder make it a little easier to issue the necessary REST requests and view the responses when using Visual Studio Code. These files require the REST Client extension for Visual Studio Code.

### Getting the Test Results
To get the test results, run the following command:

```bash
./examples/building_comfort/drasi/get_test_run_results.sh <folder_name>
```

This script will retrieve various files containing test results from the test run and store them in a local folder named `<folder_name>`. The folder will contain the following files:

| File Name | Description |
|-----------|-------------|
| source_summary.json | A summary of the Test Run Source's operation during the Test Run. |
| query_summary.json | A summary of the Test Run Query's operation during the Test Run. |
| query_profiler_summary.json | A summary of the Query Profiler's operation during the Test Run. |
| query_profiler_change_distributions.csv | Data generated by the Query Profiler that shows the distribution of time each change spent in each component of the Drasi platform. |
| query_profiler_change_rates.csv | Data generated by the Query Profiler that shows the rate at which changes were processed by each component of the Drasi platform. |
| query_profiler_viz_all_abs.png | A visualization of the Query Profiler data that shows the length of time each change spent in each component of the Drasi platform (each pixel high row represents a single change). The times are normalized to the longest time it took any change to get through the system. |
| query_profiler_viz_drasi_abs.png | Same as above but it only includes the Drasi services, not the queues between services. |
| query_profiler_viz_all_rel.png | A visualization of the Query Profiler data that shows the length of time each change spent in each component of the Drasi platform (each pixel high row represents a single change). The times are normalized to the total time it took that change to get through the system. |
| query_profiler_viz_drasi_rel.png | Same as above but it only includes the Drasi services, not the queues between services. |

### Ending the Test Run
To end the test run, run the following command:

```bash
./examples/building_comfort/drasi/end_test_run.sh
```

This script will:
1. Stop the port forwarding of the E2E Test Service.
1. Uninstall Drasi.
1. Uninstall Dapr.

This will remove everything that was installed by the `run_test_*.sh` scripts.
Once complete, you should be able to run more tests on the same Kubernetes cluster.

## Executing the Building Comfort Example in a local process
Running the Building Comfort example test locally launches the E2E Test Service from source as a local process. This can be useful for debugging and development purposes, but will **not** test a Drasi environment without additional configuration.

The files for executing the Building Comfort Test example in a local process are located in the `test-infra/e2e-test-framework/examples/building_comfort/local` folder. 

These files assume you have Rust 1.81 installed on the machine where you will launch the test.

The folder contains the following files to launch, inspect, and control the test:

| File Name | Description |
|-----------|-------------|
| run_test.sh | A shell script that runs the Building Comfort test. |
| config.json | A JSON file that contains the configuration for the test. |
| web_api_source.http | A file used by the REST Client extension for Visual Studio Code to make it easy to interact with the E2E Test Service to inspect and control the Test Run Source. |
| web_api_query.http | A file used by the REST Client extension for Visual Studio Code to make it easy to interact with the E2E Test Service to inspect and control the Test Run Query. |

### Running the Test
To run the test, open a terminal and navigate to the `test-infra/e2e-test-framework` folder.

Run the following command:

```bash
./examples/building_comfort/local/run_test.sh
```

This script will build and run the E2E Test Service and start the Building Comfort Test.

The test is configured to use a temporary local Test Repo located in `./test-infra/e2e-test-framework/examples/building_comfort/local/test_data_cache/test_repos`.

The configuration for the Building Comfort test is included as a `local_test` embedded in the `config.json` file. 

### Inspecting and Controlling the Test
You can always inspect and control the Test Run through the Test Service Web API, which will be accessible through `http://localhost:63123`. 

The `web_api_source.http` and `web_api_query.http` files in the `local` folder make it a little easier to issue the necessary REST request and view the responses when using Visual Studio Code. These files require the REST Client extension for Visual Studio Code.

### Getting the Test Results
The test results will be stored in the `./test-infra/e2e-test-framework/examples/building_comfort/local/test_data_cache/test_runs` folder.

### Ending the Test Run
Press `Ctrl-C` to stop the Test Service. This will automatically delete the local test repo and result store.






