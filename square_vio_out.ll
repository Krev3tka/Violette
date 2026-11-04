; ModuleID = '/var/folders/bd/2rr7y99s02zbt01bpjvs0czw0000gn/T//square_vio_out.c'
source_filename = "/var/folders/bd/2rr7y99s02zbt01bpjvs0czw0000gn/T//square_vio_out.c"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32"
target triple = "arm64-apple-macosx26.0.0"

%struct.VioString = type { ptr, i64, i64 }

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef double @vio_user_float64_abs(double noundef %0) local_unnamed_addr #0 {
  %2 = fcmp ult double %0, 0.000000e+00
  %3 = fneg double %0
  %4 = select i1 %2, double %3, double %0
  ret double %4
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef double @vio_user_float64_pow(double noundef %0, double noundef %1) local_unnamed_addr #0 {
  %3 = tail call double @llvm.pow.f64(double %0, double %1)
  ret double %3
}

; Function Attrs: mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare double @llvm.pow.f64(double, double) #1

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define range(i64 -1, 2) i64 @vio_user_float64_sign(double noundef %0) local_unnamed_addr #0 {
  %2 = fcmp olt double %0, 0.000000e+00
  %3 = fcmp une double %0, 0.000000e+00
  %4 = zext i1 %3 to i64
  %5 = select i1 %2, i64 -1, i64 %4
  ret i64 %5
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef zeroext i1 @vio_user_int_isEven(i64 noundef %0) local_unnamed_addr #0 {
  %2 = and i64 %0, 1
  %3 = icmp eq i64 %2, 0
  ret i1 %3
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef zeroext i1 @vio_user_int_isOdd(i64 noundef %0) local_unnamed_addr #0 {
  %2 = and i64 %0, -9223372036854775807
  %3 = icmp eq i64 %2, 1
  ret i1 %3
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef zeroext i1 @vio_user_int_isPositive(i64 noundef %0) local_unnamed_addr #0 {
  %2 = icmp sgt i64 %0, 0
  ret i1 %2
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef zeroext i1 @vio_user_int_isNegative(i64 noundef %0) local_unnamed_addr #0 {
  %2 = icmp slt i64 %0, 0
  ret i1 %2
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef zeroext i1 @vio_user_int_isZero(i64 noundef %0) local_unnamed_addr #0 {
  %2 = icmp eq i64 %0, 0
  ret i1 %2
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef i64 @vio_user_int_min(i64 noundef %0, i64 noundef %1) local_unnamed_addr #0 {
  %3 = tail call i64 @llvm.smin.i64(i64 %0, i64 %1)
  ret i64 %3
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef i64 @vio_user_int_max(i64 noundef %0, i64 noundef %1) local_unnamed_addr #0 {
  %3 = tail call i64 @llvm.smax.i64(i64 %0, i64 %1)
  ret i64 %3
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define noundef i64 @vio_user_int_clamp(i64 noundef %0, i64 noundef %1, i64 noundef %2) local_unnamed_addr #0 {
  %4 = icmp slt i64 %0, %1
  %5 = tail call i64 @llvm.smin.i64(i64 %0, i64 %2)
  %6 = select i1 %4, i64 %1, i64 %5
  ret i64 %6
}

; Function Attrs: nounwind ssp uwtable(sync)
define double @vio_user_int_toFloat(i64 noundef %0) local_unnamed_addr #2 {
  %2 = tail call double @vio_to_float(i64 noundef %0) #7
  ret double %2
}

declare double @vio_to_float(i64 noundef) local_unnamed_addr #3

; Function Attrs: nounwind ssp uwtable(sync)
define void @vio_user_int_toString(ptr dead_on_unwind noalias writable sret(%struct.VioString) align 8 %0, i64 noundef %1) local_unnamed_addr #2 {
  tail call void @vio_to_string(ptr dead_on_unwind writable sret(%struct.VioString) align 8 %0, i64 noundef %1) #7
  ret void
}

declare void @vio_to_string(ptr dead_on_unwind writable sret(%struct.VioString) align 8, i64 noundef) local_unnamed_addr #3

; Function Attrs: nounwind ssp uwtable(sync)
define i64 @vio_user_float64_toInt(double noundef %0) local_unnamed_addr #2 {
  %2 = tail call i64 @vio_float_to_int(double noundef %0) #7
  ret i64 %2
}

declare i64 @vio_float_to_int(double noundef) local_unnamed_addr #3

; Function Attrs: nounwind ssp uwtable(sync)
define void @vio_user_float64_toString(ptr dead_on_unwind noalias writable sret(%struct.VioString) align 8 %0, double noundef %1) local_unnamed_addr #2 {
  tail call void @vio_float_to_str(ptr dead_on_unwind writable sret(%struct.VioString) align 8 %0, double noundef %1) #7
  ret void
}

declare void @vio_float_to_str(ptr dead_on_unwind writable sret(%struct.VioString) align 8, double noundef) local_unnamed_addr #3

; Function Attrs: nounwind ssp uwtable(sync)
define i64 @vio_user_string_len(ptr noundef readonly captures(none) %0) local_unnamed_addr #2 {
  %2 = alloca %struct.VioString, align 8
  call void @llvm.lifetime.start.p0(i64 24, ptr nonnull %2) #7
  call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(24) %2, ptr noundef nonnull align 8 dereferenceable(24) %0, i64 24, i1 false), !tbaa.struct !6
  %3 = call i64 @vio_str_len(ptr noundef nonnull %2) #7
  call void @llvm.lifetime.end.p0(i64 24, ptr nonnull %2) #7
  ret i64 %3
}

declare i64 @vio_str_len(ptr noundef) local_unnamed_addr #3

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.start.p0(i64 immarg, ptr captures(none)) #4

; Function Attrs: mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p0.p0.i64(ptr noalias writeonly captures(none), ptr noalias readonly captures(none), i64, i1 immarg) #5

; Function Attrs: mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite)
declare void @llvm.lifetime.end.p0(i64 immarg, ptr captures(none)) #4

; Function Attrs: nounwind ssp uwtable(sync)
define zeroext i1 @vio_user_string_isEmpty(ptr noundef readonly captures(none) %0) local_unnamed_addr #2 {
  %2 = alloca %struct.VioString, align 8
  call void @llvm.lifetime.start.p0(i64 24, ptr nonnull %2) #7
  call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(24) %2, ptr noundef nonnull align 8 dereferenceable(24) %0, i64 24, i1 false)
  %3 = call i64 @vio_str_len(ptr noundef nonnull %2) #7
  call void @llvm.lifetime.end.p0(i64 24, ptr nonnull %2) #7
  %4 = icmp eq i64 %3, 0
  ret i1 %4
}

; Function Attrs: nounwind ssp uwtable(sync)
define i64 @vio_user_string_toInt(ptr noundef readonly captures(none) %0) local_unnamed_addr #2 {
  %2 = alloca %struct.VioString, align 8
  call void @llvm.lifetime.start.p0(i64 24, ptr nonnull %2) #7
  call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(24) %2, ptr noundef nonnull align 8 dereferenceable(24) %0, i64 24, i1 false), !tbaa.struct !6
  %3 = call i64 @vio_str_to_int(ptr noundef nonnull %2) #7
  call void @llvm.lifetime.end.p0(i64 24, ptr nonnull %2) #7
  ret i64 %3
}

declare i64 @vio_str_to_int(ptr noundef) local_unnamed_addr #3

; Function Attrs: nounwind ssp uwtable(sync)
define double @vio_user_string_toFloat(ptr noundef readonly captures(none) %0) local_unnamed_addr #2 {
  %2 = alloca %struct.VioString, align 8
  call void @llvm.lifetime.start.p0(i64 24, ptr nonnull %2) #7
  call void @llvm.memcpy.p0.p0.i64(ptr noundef nonnull align 8 dereferenceable(24) %2, ptr noundef nonnull align 8 dereferenceable(24) %0, i64 24, i1 false), !tbaa.struct !6
  %3 = call double @vio_str_to_float(ptr noundef nonnull %2) #7
  call void @llvm.lifetime.end.p0(i64 24, ptr nonnull %2) #7
  ret double %3
}

declare double @vio_str_to_float(ptr noundef) local_unnamed_addr #3

; Function Attrs: mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync)
define i64 @vio_user_square(i64 noundef %0) local_unnamed_addr #0 {
  %2 = sitofp i64 %0 to double
  %3 = fmul double %2, %2
  %4 = fptosi double %3 to i64
  ret i64 %4
}

; Function Attrs: nounwind ssp uwtable(sync)
define noundef i32 @main() local_unnamed_addr #2 {
  tail call void @vio_println_int(i64 noundef 36) #7
  ret i32 0
}

declare void @vio_println_int(i64 noundef) local_unnamed_addr #3

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smin.i64(i64, i64) #6

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smax.i64(i64, i64) #6

attributes #0 = { mustprogress nofree norecurse nosync nounwind ssp willreturn memory(none) uwtable(sync) "frame-pointer"="non-leaf" "no-trapping-math"="true" "probe-stack"="__chkstk_darwin" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+altnzcv,+bti,+ccdp,+ccidx,+ccpp,+complxnum,+crc,+dit,+dotprod,+flagm,+fp-armv8,+fp16fml,+fptoint,+fullfp16,+jsconv,+lse,+neon,+pauth,+perfmon,+predres,+ras,+rcpc,+rdm,+sb,+sha2,+sha3,+specrestrict,+ssbs,+v8.1a,+v8.2a,+v8.3a,+v8.4a,+v8.5a,+v8a" }
attributes #1 = { mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #2 = { nounwind ssp uwtable(sync) "frame-pointer"="non-leaf" "no-trapping-math"="true" "probe-stack"="__chkstk_darwin" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+altnzcv,+bti,+ccdp,+ccidx,+ccpp,+complxnum,+crc,+dit,+dotprod,+flagm,+fp-armv8,+fp16fml,+fptoint,+fullfp16,+jsconv,+lse,+neon,+pauth,+perfmon,+predres,+ras,+rcpc,+rdm,+sb,+sha2,+sha3,+specrestrict,+ssbs,+v8.1a,+v8.2a,+v8.3a,+v8.4a,+v8.5a,+v8a" }
attributes #3 = { "frame-pointer"="non-leaf" "no-trapping-math"="true" "probe-stack"="__chkstk_darwin" "stack-protector-buffer-size"="8" "target-cpu"="apple-m1" "target-features"="+aes,+altnzcv,+bti,+ccdp,+ccidx,+ccpp,+complxnum,+crc,+dit,+dotprod,+flagm,+fp-armv8,+fp16fml,+fptoint,+fullfp16,+jsconv,+lse,+neon,+pauth,+perfmon,+predres,+ras,+rcpc,+rdm,+sb,+sha2,+sha3,+specrestrict,+ssbs,+v8.1a,+v8.2a,+v8.3a,+v8.4a,+v8.5a,+v8a" }
attributes #4 = { mustprogress nocallback nofree nosync nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #6 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #7 = { nounwind }

!llvm.module.flags = !{!0, !1, !2, !3, !4}
!llvm.ident = !{!5}

!0 = !{i32 2, !"SDK Version", [2 x i32] [i32 27, i32 2]}
!1 = !{i32 1, !"wchar_size", i32 4}
!2 = !{i32 8, !"PIC Level", i32 2}
!3 = !{i32 7, !"uwtable", i32 1}
!4 = !{i32 7, !"frame-pointer", i32 1}
!5 = !{!"Apple clang version 21.0.0 (clang-2100.4.1.1)"}
!6 = !{i64 0, i64 8, !7, i64 8, i64 8, !12, i64 16, i64 8, !12}
!7 = !{!8, !8, i64 0}
!8 = !{!"p1 omnipotent char", !9, i64 0}
!9 = !{!"any pointer", !10, i64 0}
!10 = !{!"omnipotent char", !11, i64 0}
!11 = !{!"Simple C/C++ TBAA"}
!12 = !{!13, !13, i64 0}
!13 = !{!"long", !10, i64 0}
