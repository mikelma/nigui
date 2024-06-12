import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from scipy.interpolate import splrep, splev
from argparse import ArgumentParser


def parse_args():
    parser = ArgumentParser()
    parser.add_argument("--path", type=str, required=True)
    parser.add_argument("--channel", type=int, required=False, default=0)
    parser.add_argument("--min-freq", type=int, required=False, default=5)
    parser.add_argument("--max-freq", type=int, required=False, default=40)
    return parser.parse_args()


def moving_average(a, n=30):
    ret = np.cumsum(a, dtype=float)
    ret[n:] = ret[n:] - ret[:-n]
    return ret[n - 1:] / n


if __name__ == "__main__":
    args = parse_args()
    fname = args.path
    ch = args.channel
    min_freq, max_freq = args.min_freq, args.max_freq

    df = pd.read_csv(fname)
    xx = df[f"channel-{ch}"].values

    spectrum, freqs, _ = plt.magnitude_spectrum(xx, Fs=250)
    plt.clf()


    idx = (freqs > min_freq) & (freqs < max_freq)

    ts = moving_average(freqs[idx])
    ys = moving_average(spectrum[idx])

    n_interior_knots = 30
    qs = np.linspace(0, 1, n_interior_knots+2)[1:-1]
    knots = np.quantile(ts, qs)
    tck = splrep(ts, ys, t=knots, k=3)
    ys_smooth = splev(ts, tck)

    # x = np.log10(freqs[idx])
    # y = np.log10(spectrum[idx])

    max_idx = np.argmax(ys_smooth)
    max_f = ts[max_idx]

    ax = plt.gca()
    ax.spines['right'].set_visible(False)
    ax.spines['top'].set_visible(False)

    plt.plot(ts, ys, alpha=0.5, c="lightgray")
    plt.plot(ts, ys_smooth)
    plt.axvline(x=max_f, c="red")
    plt.text(x=max_f+1, y=0.9*max(ys), s=f"Zure alpha: {round(max_f, 2)} Hz")
    plt.xlabel("Maiztasuna (Hz)")
    plt.ylabel("Energia")
    title = fname.split("/")[-1].split(".")[0]
    plt.title(title+"\n")

    plt.savefig(title + ".png", dpi=300)
    plt.show()
