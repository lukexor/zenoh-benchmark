# Because criterion still hasn't provided great plotting customizations, or a
# stable (and undeprecated) output format...we'll just parse the raw
# output...gross

import re
import matplotlib.pyplot as plt

criterion_out = "criterion_output.txt"

# Number of messages (should match benchmark)
NUM_MESSAGES = 1000

# Get benchmark size and time from output
pattern = r"(\w+)/(\d+)\s+time:\s+\[(\d+\.\d+)\s+(ms|µs)"

# Parsed throughputs
data = {}

# Convert size and time to throughput
def calculate_throughput(message_size_bytes, time, time_unit):
    if time_unit == "ms":
        time_div = 1000
    elif time_unit == "µs":
        time_div = 1_000_000
    else:
        print(f"unhandled time unit {time_unit}")
        return

    time_s = time / time_div
    total_bytes = message_size_bytes * NUM_MESSAGES
    gb = total_bytes / 1_000_000_000
    throughput = gb / time_s
    return throughput

# Parse criterion output
with open(criterion_out, 'r') as file:
    for line in file:
        match = re.search(pattern, line)
        if match:
            transport = match.group(1)
            msg_size = int(match.group(2))
            time = float(match.group(3))
            time_unit = match.group(4)
            throughput = calculate_throughput(msg_size, time, time_unit)

            if transport not in data:
                data[transport] = {
                    "sizes": [],
                    "throughputs": []
                }

            data[transport]["sizes"].append(msg_size)
            data[transport]["throughputs"].append(throughput)

if not data:
    print("No valid data found in the Criterion output.")
else:
    plt.figure(figsize=(10, 6))

    for transport, data in data.items():
        plt.plot(data["sizes"], data["throughputs"], marker='o', label=transport)

    xticks = [32, 128, 512, 1024, 4096, 8192, 32768, 131072, 524288]
    yticks = [0.1, 0.5, 2, 8, 16, 32, 64, 128]
    plt.xscale("log")
    plt.yscale("log")
    plt.xlabel("Message Size (Bytes)")
    plt.xticks(xticks, labels=[f"{x}" for x in xticks])
    plt.ylabel("Throughput (GB/s)")
    plt.yticks(yticks,  labels=[f"{y}" for y in yticks])
    plt.title("Throughput vs. Message Size")
    plt.grid(True)
    plt.legend()

    plt.show()
