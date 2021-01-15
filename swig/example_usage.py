
import example

# We can make std::vectors.
my_list = example.vectori([1, 2, 3, 4])
print(example.sum_up_list(my_list))

# We can pass in strings.
my_data = example.Data("This is an example string.")
print(my_data)
print("Number of a:", my_data.count_of_letters[ord("a")])
my_data.do_a_thing()
print("Number of a * 2:", my_data.count_of_letters[ord("a")])


