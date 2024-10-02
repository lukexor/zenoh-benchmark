benchmark:
  cargo bench | tee criterion_output.txt
  python3 graph-throughput.py
