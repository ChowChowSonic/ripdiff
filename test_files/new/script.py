def process_data(items, reverse=False):
    results = []
    for item in items:
        if reverse:
            results.append(item[::-1])
        else:
            results.append(item.upper())
    return results

def main():
    data = ["hello", "world", "test", "new_item"]
    output = process_data(data, reverse=True)
    for item in output:
        print(f"Item: {item}")

if __name__ == "__main__":
    main()
