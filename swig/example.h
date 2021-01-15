
#pragma once

#include <string>
#include <vector>

int sum_up_list(std::vector<int> l);

struct Data {
	std::string	input;
	std::vector<int> count_of_letters;

	// Constructor
	Data(std::string _input);

	void do_a_thing();
};

