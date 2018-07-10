# vmaf-plot

 Plot graphs from JSON files produced by libvmaf (VMAF/MS_SSIM/PSNR).

 Plots show metric scores per frame, not per video. A line represents
 a single video, not multiple encdoings using the same encoder.

 One way to obtain those files is by using the `vmafossexec` with
 the arguments `--psnr --ms-ssim --log-fmt json`. The tool is usually
 packaged with `libvmaf`.

 `gnuplot` is used for plotting, with `PNG` files as output.

