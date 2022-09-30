
import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
import sys
import argparse

# Very basic script to use matplotlib to plot raw data saved from psrec
# Designed (currently) to only use bare minimal libs (python + matplotlib)

# TODO: Make this more pythonic (PEP8), and robust (error handling)

# TODO: we could do the pre-processing in the Rust implementation before saving to be more efficient,
#       although we'd have to save it as comment metadata in primitive file formats (CSV)...

def readDataValuesFromCSVFile(filename):
    timeValues = []
    cpuValues = []
    rssValues = []

    timeUnit = "s"
    rssUnit = "mb"

    fData = open(filename, "r")
    for line in fData:
        if len(line) == 0:
            continue
        if line[0] == '#':
            continue
        # is this a good idea? Might be better to error...
        if not ',' in line:
            continue

        items = line.split(',')
        time = items[0]
        cpu = items[1]
        rss = items[2]

        rssVal = float(rss) / 1024.0 / 1024.0
        if rssVal > 4000.0:
            rssUnit = "gb"

        timeValues.append(float(time))
        cpuValues.append(float(cpu))
        rssValues.append(rssVal)
    
    # See what last time was, and if > 5 mins, use hours/minutes instead of seconds as the units
    if len(timeValues) > 0:
        if timeValues[-1] > (60.0 * 60.0 * 5.0):
            # Use hours
            timeUnit = "h"
            # Also resize the numbers
            for i in range(len(timeValues)):
                timeValues[i] /= 3600.0
        elif timeValues[-1] > (60.0 * 5.0):
            # Use minutes
            timeUnit = "m"
            # Also resize the numbers
            for i in range(len(timeValues)):
                timeValues[i] /= 60.0
    
    if rssUnit == "gb":
        # Resize numbers to GB size
        # TODO: could use numpy for this
        for i in range(len(rssValues)):
            rssValues[i] /= 1024.0
    
    # TODO: work out how to handle absolute (non-normalised) CPU time values... it's not
    #       clear how to know that from just the values themselves, so we may need to use comment
    #       metadata (or a better format! :) )
    
    values = {'tv':timeValues, 'cv': cpuValues, 'rv': rssValues, 'tu':timeUnit, 'ru':rssUnit}
    return values

def generateBasicCombinedPlot(dataValues, areaPlot):
    fig, ax1 = plt.subplots(1, 1)

    fig.tight_layout()
    fig.set_figwidth(15)
    fig.set_figheight(8)

    # Colours for axis labels (which are darker than the plot colours), so we don't need to bother
    # with legends...
    CPU_AXIS_COLOUR = "#0000bf"
    RSS_AXIS_COLOUR = "#bf0000"

    timeValues = dataValues['tv']

    ax2 = ax1.twinx()
    ax1.set_title('Process recording (CPU usage and RSS memory usage)')
    xLabel = "Time elapsed ({})".format("Minutes" if dataValues['tu'] == "m" else "Hours" if dataValues['tu'] == "h" else "Seconds")
    ax1.set_xlabel(xLabel)
    if areaPlot:
        ax1.fill_between(timeValues, dataValues['cv'], color='blue', alpha=0.6)
        ax2.fill_between(timeValues, dataValues['rv'], color='red', alpha=0.6)
    else:    
        ax1.plot(timeValues, dataValues['cv'], color='blue')
        ax2.plot(timeValues, dataValues['rv'], color='red')

    ax1.set_ylabel('CPU usage (normalised %)', color=CPU_AXIS_COLOUR)
    rssYLabel = "Memory RSS ({})".format("MB" if dataValues['ru'] == "mb" else "GB")
    ax2.set_ylabel(rssYLabel, color=RSS_AXIS_COLOUR)
    ax2.get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
    # ax2.get_yaxis().set_minor_locator(mpl.ticker.AutoMinorLocator())
    # for tick in ax2.get_yaxis().get_major_ticks():
    #     tick.label1.set_color(RSS_AXIS_COLOUR)
#    ax1.legend(['CPU Usage'])
#    ax2.legend(['RSS'])
    
    ax1.yaxis.grid(color='lightgray')

    fig.tight_layout()

    # set_xmargin() doesn't seem to work (or do what I thought it would?)...
    ax1.set_ylim(ymin=0, ymax=100.0)
    ax1.set_xlim(xmin=0, xmax=timeValues[-1])
    ax2.set_ylim(ymin=0, ymax=max(dataValues['rv']))
    ax2.set_xlim(xmin=0, xmax=timeValues[-1])
    plt.show()

def generateBasicSeparatePlot(dataValues, areaPlot):
    fig, (ax1, ax2) = plt.subplots(2, 1)

    fig.tight_layout()
    fig.set_figwidth(15)
    fig.set_figheight(8)

    fig.suptitle('Process recording (CPU usage and RSS memory usage)')

    timeValues = dataValues['tv']

    ax1.yaxis.grid(color='lightgray')
    if areaPlot:
        ax1.fill_between(timeValues, dataValues['cv'], color='blue', alpha=0.7)
    else:
        ax1.plot(timeValues, dataValues['cv'], color='blue')
    xLabel = "Time elapsed ({})".format("Minutes" if dataValues['tu'] == "m" else "Hours" if dataValues['tu'] == "h" else "Seconds")
    ax1.set_xlabel(xLabel)
    ax1.set_ylabel('CPU usage (normalised %)')
    
    ax2.yaxis.grid(color='lightgray')
    if areaPlot:
        ax2.fill_between(timeValues, dataValues['rv'], color='red', alpha=0.7)
    else:
        ax2.plot(timeValues, dataValues['rv'], color='red')
    ax2.set_xlabel(xLabel)

    rssYLabel = "Memory RSS ({})".format("MB" if dataValues['ru'] == "mb" else "GB")
    ax2.set_ylabel(rssYLabel)
    ax2.get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
    #ax2.get_yaxis().set_minor_locator(mpl.ticker.AutoMinorLocator())

    # set_xmargin() doesn't seem to work (or do what I thought it would?)...
    ax1.set_ylim(ymin=0, ymax=100.0)
    ax1.set_xlim(xmin=0, xmax=timeValues[-1])
    ax2.set_ylim(ymin=0, ymax=max(dataValues['rv']))
    ax2.set_xlim(xmin=0, xmax=timeValues[-1])

    fig.tight_layout()

    plt.show()

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Error: no input file provided.")
        exit(-1)

    parser = argparse.ArgumentParser()
    parser.add_argument("inputFile", help="The input filename containing the raw data recording to plot.")
    parser.add_argument("--combined", action='store_true', help="Plot the CPU and Memory RSS values in a combined single plot.")
    parser.add_argument("--areaplot", action='store_true', help="Plot the values as solid areas, rather than line plots.")

    args = parser.parse_args()

    dataValues = readDataValuesFromCSVFile(args.inputFile)
    if args.combined:
        generateBasicCombinedPlot(dataValues, args.areaplot)
    else:
        generateBasicSeparatePlot(dataValues, args.areaplot)
    
