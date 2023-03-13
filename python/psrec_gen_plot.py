
'''
 psrec
 Copyright 2022-2023 Peter Pearson.
 Licensed under the Apache License, Version 2.0 (the "License");
 You may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 http://www.apache.org/licenses/LICENSE-2.0
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
 ---------
'''

import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
import sys
import argparse

# *Very* basic script to use matplotlib to plot raw data saved from psrec
# Designed (currently) to only use bare minimal libs (python + matplotlib)

# TODO: Make this more pythonic (PEP8), and robust (error handling)

# TODO: we could do the pre-processing in the Rust implementation before saving to be more efficient,
#       although we'd have to save it as comment metadata in primitive file formats (CSV)...

def readDataValuesFromCSVFile(filename):
    timeValues = []
    cpuValues = []
    rssValues = []
    # Note: this is optional
    threadCountsValues = []

    timeUnit = "s"
    rssUnit = "mb"

    maxCPUValue = 0.0

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

        cpu = float(cpu)

        # keep track of maximum cpu value, so we can later try and see if we can work
        # if the values might be normalised or not (not completely robustly though...)
        if cpu > maxCPUValue:
            maxCPUValue = cpu

        rssVal = float(rss) / 1024.0 / 1024.0
        if rssVal > 4000.0:
            rssUnit = "gb"

        timeValues.append(float(time))
        cpuValues.append(cpu)
        rssValues.append(rssVal)

        if len(items) > 3:
            threadCountsValues.append(int(items[3]))
    
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
        
    values = {'tv':timeValues, 'cv':cpuValues, 'rv':rssValues, 'tcv':threadCountsValues, 'tu':timeUnit, 'ru':rssUnit, 'mcv':maxCPUValue}
    return values

def generateBasicCombinedPlot(dataValues, areaPlot, verticalGridLines):
    fig, ax1 = plt.subplots(1, 1)

    fig.tight_layout()
    fig.set_figwidth(15)
    fig.set_figheight(8)

    # Colours for axis labels (which are darker than the plot colours), so we don't need to bother
    # with legends...
    CPU_AXIS_COLOUR = "#0000bf"
    RSS_AXIS_COLOUR = "#bf0000"

    timeValues = dataValues['tv']

    # this obviously isn't going to be completely robust, but on the assumption there'll be some values using more than
    # one core, works fairly well in practice
    isCPUDataPossiblyAbsolute = dataValues['mcv'] > 105.0

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

    ax1.set_ylabel('CPU usage ({} %)'.format("absolute" if isCPUDataPossiblyAbsolute else "normalised"), color=CPU_AXIS_COLOUR)
    ax1.get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))

    rssYLabel = "Memory RSS ({})".format("MB" if dataValues['ru'] == "mb" else "GB")
    ax2.set_ylabel(rssYLabel, color=RSS_AXIS_COLOUR)
    ax2.get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
    
    ax1.yaxis.grid(color='lightgray')
    if verticalGridLines:
        ax1.xaxis.grid(color='lightgray')

    fig.tight_layout()

    # set_xmargin() doesn't seem to work (or do what I thought it would?)...
    if isCPUDataPossiblyAbsolute:
        ax1.set_ylim(ymin=0, ymax=dataValues['mcv'])
    else:
        ax1.set_ylim(ymin=0, ymax=101.0)
    ax1.set_xlim(xmin=0, xmax=timeValues[-1])
    ax2.set_ylim(ymin=0, ymax=None)
    ax2.set_xlim(xmin=0, xmax=timeValues[-1])
    plt.show()

def generateBasicSeparatePlot(dataValues, areaPlot, verticalGridLines):
    haveThreadCounts = len(dataValues['tcv']) > 0

    fig = None
    axes = None

    if haveThreadCounts:
        fig, axes = plt.subplots(3, 1)
    else:
        fig, axes = plt.subplots(2, 1)

    fig.tight_layout()
    fig.set_figwidth(15)
    fig.set_figheight(8)

    if haveThreadCounts:
        fig.suptitle('Process recording (CPU usage, RSS memory usage and Thread Count)')
    else:
        fig.suptitle('Process recording (CPU usage and RSS memory usage)')

    timeValues = dataValues['tv']

    # this obviously isn't going to be completely robust, but on the assumption there'll be some values using more than
    # one core, works fairly well in practice
    isCPUDataPossiblyAbsolute = dataValues['mcv'] > 105.0

    axes[0].yaxis.grid(color='lightgray')
    if verticalGridLines:
        axes[0].xaxis.grid(color='lightgray')
    if areaPlot:
        axes[0].fill_between(timeValues, dataValues['cv'], color='blue', alpha=0.7)
    else:
        axes[0].plot(timeValues, dataValues['cv'], color='blue')
    xLabel = "Time elapsed ({})".format("Minutes" if dataValues['tu'] == "m" else "Hours" if dataValues['tu'] == "h" else "Seconds")
    axes[0].set_xlabel(xLabel)
    axes[0].set_ylabel('CPU usage ({} %)'.format("absolute" if isCPUDataPossiblyAbsolute else "normalised"))
    axes[0].get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
    
    axes[1].yaxis.grid(color='lightgray')
    if verticalGridLines:
        axes[1].xaxis.grid(color='lightgray')
    if areaPlot:
        axes[1].fill_between(timeValues, dataValues['rv'], color='red', alpha=0.7)
    else:
        axes[1].plot(timeValues, dataValues['rv'], color='red')
    axes[1].set_xlabel(xLabel)

    rssYLabel = "Memory RSS ({})".format("MB" if dataValues['ru'] == "mb" else "GB")
    axes[1].set_ylabel(rssYLabel)
    axes[1].get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))
    #axes[1].get_yaxis().set_minor_locator(mpl.ticker.AutoMinorLocator())

    if haveThreadCounts:
        axes[2].yaxis.grid(color='lightgray')
        if verticalGridLines:
            axes[2].xaxis.grid(color='lightgray')
        if areaPlot:
            axes[2].fill_between(timeValues, dataValues['tcv'], color='green', alpha=0.7)
        else:
            axes[2].plot(timeValues, dataValues['tcv'], color='green')
        axes[2].set_xlabel(xLabel)

        threadsYLabel = "Active Thread Count"
        axes[2].set_ylabel(threadsYLabel)
        axes[2].get_yaxis().set_major_formatter(mpl.ticker.FuncFormatter(lambda x, p: format(int(x), ',')))

    # set_xmargin() doesn't seem to work (or do what I thought it would?)...
    if isCPUDataPossiblyAbsolute:
        axes[0].set_ylim(ymin=0, ymax=dataValues['mcv'])
    else:
        axes[0].set_ylim(ymin=0, ymax=101.0)
    axes[0].set_xlim(xmin=0, xmax=timeValues[-1])
    axes[1].set_ylim(ymin=0, ymax=None)
    axes[1].set_xlim(xmin=0, xmax=timeValues[-1])

    if haveThreadCounts:
        axes[2].set_xlim(xmin=0, xmax=None)

    fig.tight_layout()

    plt.show()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(
                    prog='psrec generate plot',
                    description='Draws a plot of the data the main psrec program recorded, using Python and matplotlib',)
    
    parser.add_argument("inputFile", help="The input filename containing the raw data recording to plot.")
    parser.add_argument("--combined", action='store_true', help="Plot the recorded values in a combined single plot.")
    parser.add_argument("--areaplot", action='store_true', help="Plot the values as solid areas, rather than line plots.")
    parser.add_argument("--verticalgrid", action='store_true', help="Draw vertical grid lines for the Time axis.")

    args = parser.parse_args()

    dataValues = readDataValuesFromCSVFile(args.inputFile)
    if args.combined:
        generateBasicCombinedPlot(dataValues, args.areaplot, args.verticalgrid)
    else:
        generateBasicSeparatePlot(dataValues, args.areaplot, args.verticalgrid)
    
