package main

import (
	"os";
	"io";
	"bufio";
	"fmt";
	"strconv";
	"strings";
)

func main() {
	reader := bufio.NewReader(os.Stdin)

	sum := 0

	for {
		text, err := reader.ReadString('\n')
		if (err == io.EOF) {
			break;
		}
		if (err != nil) {
			panic(err)
		}

		mass, err := strconv.Atoi(strings.Trim(text, "\n"))
		if (err != nil) {
			panic(err);
		}

		sum = sum + ((mass / 3) - 2)
	}

	fmt.Printf("%d\n", sum)
}
