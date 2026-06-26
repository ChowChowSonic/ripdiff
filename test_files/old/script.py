def process_data(items):
    results = []
    for item in items:
        results.append(item.upper())
    return results

def main():
    data = ["hello", "world", "test"]
    output = process_data(data)
    for item in output:
        print(item)

if __name__ == "__main__":
    main()
