def eval_eq(first, second, third, fourth, fifth):
    return first + second * third ** 2 + fourth ** 3 - fifth == 399


options = {2, 3, 5, 7, 9}

used = set()
used_list = []


def solve_eq():
    if len(used) == 5:
        return eval_eq(*used_list)

    my_options = options - used
    for option in my_options:
        used.add(option)
        used_list.append(option)
        if solve_eq():
            return True
        else:
            used_list.pop()
            used.remove(option)


assert solve_eq()
print(used_list)
