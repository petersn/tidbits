
#include "example.h"

int sum_up_list(std::vector<int> l) {
	int sum = 0;
	for (auto x : l)
		sum += x;
	return sum;
}

Data::Data(std::string _input) {
	input = _input;
	for (int i = 0; i < 256; i++) {
		int count = 0;
		for (char c : input)
			count += c == i;
		count_of_letters.push_back(count);
	}
}

void Data::do_a_thing() {
	for (int& value : count_of_letters)
		value *= 2;
}

