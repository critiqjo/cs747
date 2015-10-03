#!/bin/env python2

# ------ Params ------ #

Rmin = -5
Rmax = 3

Ns = 20
Na = 40

gamma = 0.9

# -------------------- #

from random import uniform

print Ns
print Na

for s in range(0, Ns):
    for a in range(0, Na):
        for s_ in range(0, Ns):
            print str(uniform(Rmin, Rmax)) + "\t",

        print "\n",

for s in range(0, Ns):
    for a in range(0, Na):
        T_sa = []
        for s_ in range(0, Ns):
            T_sa.append(uniform(0, 1))
        den = sum(T_sa)
        T_sa = [i/den for i in T_sa]
        for t_sas_ in T_sa:
            print str(t_sas_) + "\t",

        print "\n",

print gamma
