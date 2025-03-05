import subprocess

if __name__ == '__main__':
    # 0. Update submodules
    try:
        # subprocess.run(['git', 'submodule', 'update', '--init', '--recursive'])
        # 1. Generate Halide rewrites using Chompy
        chompy_rw_proc = subprocess.run(['cargo', 'test', '--test', 'halide', '--', '--nocapture'], cwd='chompy', stdout=subprocess.PIPE, text=True)
        for line in chompy_rw_proc.stdout:
            print(line, end='')
            if "has been running" in line:
                chompy_rw_proc.kill()
                break
    except Exception as e:
        print(e)

    print("here was the output of the chompy test")
    print(chompy_rw_proc.stdout)
    
