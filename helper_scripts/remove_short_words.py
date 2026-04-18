file_input = open("./common_english_words.txt", mode="r")
file_output = open("./common_english_words_long.txt", mode="w")

for line in file_input:
    if len(line) > 3:
        file_output.write(line)