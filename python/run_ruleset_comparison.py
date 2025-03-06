import argparse
import subprocess
import os

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Run small subset of the Caviar evaluation given a ruleset')
    parser.add_argument('--chompy-ruleset-path', type=str, help='Path to the Chompy ruleset')
    parser.add_argument('--dataset-path', type=str, help='Path to the dataset to evaluate on')
    parser.add_argument('--output-path', type=str, help='Path to output the ruleset comparison JSON to')

    chompy_eval_dir = os.environ.get('CHOMPY_EVAL_DIR')


    if chompy_eval_dir is None:
        raise Exception("CHOMPY_EVAL_DIR environment variable not set")

    args = parser.parse_args()
    chompy_rule_path = os.path.abspath(args.chompy_ruleset_path)
    output_path = os.path.abspath(args.output_path)
    dataset_path = os.path.abspath(args.dataset_path)

    subprocess.run(['cargo', 'run', '--release', '--', '--eval-mode', 'caviar',
                              '--dataset-path', args.dataset_path,
                              '--chompy-ruleset-path', chompy_rule_path, '--ruleset-comparison-output-path',
                              output_path], cwd=f"{chompy_eval_dir}/eval/")
