import time



start_time = time.time()



def triangle_pattern_stars(n, base_length):

    """

    Generates a triangle pattern stars with n stars, each with base length base_length.

    """

    print("Generating triangle pattern stars...")

    stars = []

    for i in range(n):

        star = []

        for j in range(n):

            if i == j:

                star.append("*")

            else:

                star.append(" ")

        stars.append(star)

    for i in range(n):

        print("".join(stars[i]))

    print("Finished in %.3f seconds." % (time.time() - start_time))



triangle_pattern_stars(10, 2)