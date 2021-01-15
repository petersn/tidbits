// Swig header

// If you want to use certain C++ standard library types you can tell SWIG that you want to use them, and it'll emit code to help you interoperate them between C++ and Python.
%include <stdint.i>
%include <std_string.i>
%include <std_vector.i>

%module example %{
    #include "example.h"
%}

%include "example.h"

// If you want to use any template instantiations you have to explicitly list the ones you want to use.
namespace std {
    %template(vectori) vector<int>;
}

