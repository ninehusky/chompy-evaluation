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
        for result in json_data['results']:
            print(f"expression: {result['expression']}")
            print(f"Chompy result: {result['chompy_result']['stop_reason']}")
            print(f"Caviar result: {result['caviar_result']['stop_reason']}")
            if 'Matched' in result['chompy_result']['stop_reason']:
                chompy_successes += 1

            if 'Matched' in result['caviar_result']['stop_reason']:
                caviar_successes += 1

        print(f"Chompy successes: {chompy_successes}")
        print(f"Caviar successes: {caviar_successes}")





