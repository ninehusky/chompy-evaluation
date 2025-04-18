"""
This script filters a results file from the `caviar` step of the evaluation to generate
a Caviar-friendly CSV file which only contains expressions which require conditional
rewrite rules.
"""

import json
import argparse

EXPECTED_DISCREPANCY_COUNT = 688

def filter_results(json_file):
    def result_is_success(result):
        stop_reason = result['stop_reason']
        return stop_reason == "Goal 0 Matched" or stop_reason == "Goal 1 Matched" or "Impossible" in stop_reason

    with open(json_file, 'r') as file:
        data = json.load(file)
        results = []
        count = 0
        for i, result in enumerate(data['results']):
            if result_is_success(result['caviar_result']) and not result_is_success(result['other_result']):
                print(result['caviar_result']['start_expression'])
                # ID,Expression,HalideResult,HalideTime
                count += 1
                results.append((i, result['caviar_result']['start_expression'], result['caviar_result']['halide_result'], result['caviar_result']['halide_time']))
        
        print(f"Total number of results: {count}")
        assert len(results) == EXPECTED_DISCREPANCY_COUNT, f"Expected {EXPECTED_DISCREPANCY_COUNT} results, but got {len(results)}"
        return results



if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Filter JSON entries based on specific conditions.")
    parser.add_argument("json_file", help="Path to the JSON file to process.")
    parser.add_argument("output_file", default="filtered_results.csv", help="Path to save the filtered results.")
    args = parser.parse_args()
    
    results = filter_results(args.json_file)

    with open(args.output_file, 'w') as file:
        file.write("ID,Expression,HalideResult,HalideTime\n")
        for result in results:
            file.write(f"{result[0]},{result[1]},{result[2]},{result[3]}\n")