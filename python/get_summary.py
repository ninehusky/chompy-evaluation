import argparse
import json


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Get summary of evaluation file generated by running `eval` with `--eval-mode caviar`')
    parser.add_argument('--filename', type=str, help='Name of the file to summarize')

    args = parser.parse_args()

    with open(args.filename) as f:
        json_data = json.load(f)
        chompy_successes = 0
        caviar_successes = 0
        chompy_inconsistencies = {}
        caviar_inconsistencies = {}
        disagreements = 0


        chompy_failure_data = {}
        chompy_failure_caviar_successes = 0
        caviar_failure_chompy_successes = 0
        for result in json_data['results']:
            chompy_stop_reason = result['chompy_result']['stop_reason']
            caviar_stop_reason = result['caviar_result']['stop_reason']

            chompy_success = False
            caviar_success = False

            if "Goal 0" in chompy_stop_reason and result['z3_result'].strip() != "invalid":
                chompy_inconsistencies['Goal 0'] = chompy_inconsistencies.get('Goal 0', 0) + 1
            elif "Goal 1" in chompy_stop_reason and result['z3_result'].strip() != "valid":
                chompy_inconsistencies['Goal 1'] = chompy_inconsistencies.get('Goal 1', 0) + 1
            elif "Impossible" in chompy_stop_reason and result['z3_result'].strip() != "unknown":
                chompy_inconsistencies['Unknown'] = chompy_inconsistencies.get('Unknown', 0) + 1
                print("expr: ", result['expression'])
                print("our stop reason:", chompy_stop_reason)
                print("z3 result: ", result['z3_result'])

            if "Goal 0" in caviar_stop_reason and result['z3_result'] != "invalid":
                caviar_inconsistencies['Goal 0'] = caviar_inconsistencies.get('Goal 0', 0) + 1
                print("expr: ", result['expression'])
                print("their stop reason:", caviar_stop_reason)
                print("z3 result: ", result['z3_result'])
            elif "Goal 1" in caviar_stop_reason and result['z3_result'] != "valid":
                caviar_inconsistencies['Goal 1'] = caviar_inconsistencies.get('Goal 1', 0) + 1
                print("expr: ", result['expression'])
                print("their stop reason:", caviar_stop_reason)
                print("z3 result: ", result['z3_result'])
            elif "Impossible" in caviar_stop_reason and result['z3_result'] != "unknown":
                caviar_inconsistencies['Unknown'] = caviar_inconsistencies.get('Unknown', 0) + 1
                print("expr: ", result['expression'])
                print("their stop reason:", caviar_stop_reason)
                print("z3 result: ", result['z3_result'])

            if "Goal 0" in chompy_stop_reason and "Goal 1" in caviar_stop_reason or "Goal 1" in chompy_stop_reason and "Goal 0" in caviar_stop_reason or \
                    "Goal" in chompy_stop_reason and "Impossible" in caviar_stop_reason or "Impossible" in chompy_stop_reason and "Goal" in caviar_stop_reason:
                print("expression: ", result['expression'])
                print("chompy reason: ", chompy_stop_reason)
                print("caviar reason: ", caviar_stop_reason)
                print("z3 result", result['z3_result'])
                disagreements += 1

            if 'Matched' in chompy_stop_reason or 'Impossible' in chompy_stop_reason:
                chompy_success = True
                chompy_successes += 1
            else:
                # chompy failed
                failure_reason = "Time Limit" if "Time Limit" in chompy_stop_reason else \
                    "Node Limit" if "Node Limit" in chompy_stop_reason else \
                    chompy_stop_reason
                chompy_failure_data[failure_reason] = chompy_failure_data.get(failure_reason, 0) + 1


            if 'Matched' in caviar_stop_reason or 'Impossible' in caviar_stop_reason:
                caviar_success = True
                caviar_successes += 1

            if chompy_success and not caviar_success:
                caviar_failure_chompy_successes += 1
            elif caviar_success and not chompy_success:
                chompy_failure_caviar_successes += 1


        print(f"Chompy successes: {chompy_successes}")
        print(f"Caviar successes: {caviar_successes}")
        print(f"Chompy failures, Caviar successes: {chompy_failure_caviar_successes}")
        print(f"Caviar failures, Chompy successes: {caviar_failure_chompy_successes}")
        print("Chompy failure data:")
        for reason, count in chompy_failure_data.items():
            print(f"{reason}: {count}")

        if not chompy_inconsistencies:
            print("No chompy inconsistencies found")
        else:
            print("Chompy inconsistencies:")
            for goal, count in chompy_inconsistencies.items():
                print(f"{goal}: {count}")

        if not caviar_inconsistencies:
            print("No caviar inconsistencies found")
        else:
            print("Caviar inconsistencies:")
            for goal, count in caviar_inconsistencies.items():
                print(f"{goal}: {count}")

        if not disagreements:
            print("No disagreements found")
        else:
            print(f"Disagreements: {disagreements}")
