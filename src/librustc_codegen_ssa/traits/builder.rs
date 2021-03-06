// Copyright 2018 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::abi::AbiBuilderMethods;
use super::asm::AsmBuilderMethods;
use super::debuginfo::DebugInfoBuilderMethods;
use super::intrinsic::IntrinsicCallMethods;
use super::type_::ArgTypeMethods;
use super::HasCodegen;
use common::{AtomicOrdering, AtomicRmwBinOp, IntPredicate, RealPredicate, SynchronizationScope};
use mir::operand::OperandRef;
use mir::place::PlaceRef;
use rustc::ty::layout::{Align, Size};
use std::ffi::CStr;
use MemFlags;

use std::borrow::Cow;
use std::ops::Range;
use syntax::ast::AsmDialect;

pub trait BuilderMethods<'a, 'tcx: 'a>:
    HasCodegen<'tcx>
    + DebugInfoBuilderMethods<'tcx>
    + ArgTypeMethods<'tcx>
    + AbiBuilderMethods<'tcx>
    + IntrinsicCallMethods<'tcx>
    + AsmBuilderMethods<'tcx>
{
    fn new_block<'b>(cx: &'a Self::CodegenCx, llfn: Self::Value, name: &'b str) -> Self;
    fn with_cx(cx: &'a Self::CodegenCx) -> Self;
    fn build_sibling_block<'b>(&self, name: &'b str) -> Self;
    fn cx(&self) -> &Self::CodegenCx;
    fn llfn(&self) -> Self::Value;
    fn llbb(&self) -> Self::BasicBlock;
    fn count_insn(&self, category: &str);

    fn set_value_name(&mut self, value: Self::Value, name: &str);
    fn position_at_end(&mut self, llbb: Self::BasicBlock);
    fn position_at_start(&mut self, llbb: Self::BasicBlock);
    fn ret_void(&mut self);
    fn ret(&mut self, v: Self::Value);
    fn br(&mut self, dest: Self::BasicBlock);
    fn cond_br(
        &mut self,
        cond: Self::Value,
        then_llbb: Self::BasicBlock,
        else_llbb: Self::BasicBlock,
    );
    fn switch(
        &mut self,
        v: Self::Value,
        else_llbb: Self::BasicBlock,
        num_cases: usize,
    ) -> Self::Value;
    fn invoke(
        &mut self,
        llfn: Self::Value,
        args: &[Self::Value],
        then: Self::BasicBlock,
        catch: Self::BasicBlock,
        funclet: Option<&Self::Funclet>,
    ) -> Self::Value;
    fn unreachable(&mut self);
    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fadd_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn sub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fsub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fsub_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn mul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fmul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fmul_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn udiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn exactudiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn sdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn exactsdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fdiv_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn urem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn srem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn frem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn frem_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn shl(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn lshr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn ashr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn and(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn or(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn xor(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn neg(&mut self, v: Self::Value) -> Self::Value;
    fn fneg(&mut self, v: Self::Value) -> Self::Value;
    fn not(&mut self, v: Self::Value) -> Self::Value;

    fn alloca(&mut self, ty: Self::Type, name: &str, align: Align) -> Self::Value;
    fn dynamic_alloca(&mut self, ty: Self::Type, name: &str, align: Align) -> Self::Value;
    fn array_alloca(
        &mut self,
        ty: Self::Type,
        len: Self::Value,
        name: &str,
        align: Align,
    ) -> Self::Value;

    fn load(&mut self, ptr: Self::Value, align: Align) -> Self::Value;
    fn volatile_load(&mut self, ptr: Self::Value) -> Self::Value;
    fn atomic_load(&mut self, ptr: Self::Value, order: AtomicOrdering, size: Size) -> Self::Value;
    fn load_operand(&mut self, place: PlaceRef<'tcx, Self::Value>)
        -> OperandRef<'tcx, Self::Value>;

    fn range_metadata(&mut self, load: Self::Value, range: Range<u128>);
    fn nonnull_metadata(&mut self, load: Self::Value);

    fn store(&mut self, val: Self::Value, ptr: Self::Value, align: Align) -> Self::Value;
    fn store_with_flags(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        align: Align,
        flags: MemFlags,
    ) -> Self::Value;
    fn atomic_store(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        order: AtomicOrdering,
        size: Size,
    );

    fn gep(&mut self, ptr: Self::Value, indices: &[Self::Value]) -> Self::Value;
    fn inbounds_gep(&mut self, ptr: Self::Value, indices: &[Self::Value]) -> Self::Value;
    fn struct_gep(&mut self, ptr: Self::Value, idx: u64) -> Self::Value;

    fn trunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn sext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn fptoui(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn fptosi(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn uitofp(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn sitofp(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn fptrunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn fpext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn ptrtoint(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn inttoptr(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn bitcast(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;
    fn intcast(&mut self, val: Self::Value, dest_ty: Self::Type, is_signed: bool) -> Self::Value;
    fn pointercast(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;

    fn icmp(&mut self, op: IntPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn fcmp(&mut self, op: RealPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value;

    fn empty_phi(&mut self, ty: Self::Type) -> Self::Value;
    fn phi(
        &mut self,
        ty: Self::Type,
        vals: &[Self::Value],
        bbs: &[Self::BasicBlock],
    ) -> Self::Value;
    fn inline_asm_call(
        &mut self,
        asm: &CStr,
        cons: &CStr,
        inputs: &[Self::Value],
        output: Self::Type,
        volatile: bool,
        alignstack: bool,
        dia: AsmDialect,
    ) -> Option<Self::Value>;

    fn memcpy(
        &mut self,
        dst: Self::Value,
        dst_align: Align,
        src: Self::Value,
        src_align: Align,
        size: Self::Value,
        flags: MemFlags,
    );
    fn memmove(
        &mut self,
        dst: Self::Value,
        dst_align: Align,
        src: Self::Value,
        src_align: Align,
        size: Self::Value,
        flags: MemFlags,
    );
    fn memset(
        &mut self,
        ptr: Self::Value,
        fill_byte: Self::Value,
        size: Self::Value,
        align: Align,
        flags: MemFlags,
    );

    fn minnum(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn maxnum(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value;
    fn select(
        &mut self,
        cond: Self::Value,
        then_val: Self::Value,
        else_val: Self::Value,
    ) -> Self::Value;

    fn va_arg(&mut self, list: Self::Value, ty: Self::Type) -> Self::Value;
    fn extract_element(&mut self, vec: Self::Value, idx: Self::Value) -> Self::Value;
    fn insert_element(
        &mut self,
        vec: Self::Value,
        elt: Self::Value,
        idx: Self::Value,
    ) -> Self::Value;
    fn shuffle_vector(
        &mut self,
        v1: Self::Value,
        v2: Self::Value,
        mask: Self::Value,
    ) -> Self::Value;
    fn vector_splat(&mut self, num_elts: usize, elt: Self::Value) -> Self::Value;
    fn vector_reduce_fadd_fast(&mut self, acc: Self::Value, src: Self::Value) -> Self::Value;
    fn vector_reduce_fmul_fast(&mut self, acc: Self::Value, src: Self::Value) -> Self::Value;
    fn vector_reduce_add(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_mul(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_and(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_or(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_xor(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_fmin(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_fmax(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_fmin_fast(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_fmax_fast(&mut self, src: Self::Value) -> Self::Value;
    fn vector_reduce_min(&mut self, src: Self::Value, is_signed: bool) -> Self::Value;
    fn vector_reduce_max(&mut self, src: Self::Value, is_signed: bool) -> Self::Value;
    fn extract_value(&mut self, agg_val: Self::Value, idx: u64) -> Self::Value;
    fn insert_value(&mut self, agg_val: Self::Value, elt: Self::Value, idx: u64) -> Self::Value;

    fn landing_pad(
        &mut self,
        ty: Self::Type,
        pers_fn: Self::Value,
        num_clauses: usize,
    ) -> Self::Value;
    fn add_clause(&mut self, landing_pad: Self::Value, clause: Self::Value);
    fn set_cleanup(&mut self, landing_pad: Self::Value);
    fn resume(&mut self, exn: Self::Value) -> Self::Value;
    fn cleanup_pad(&mut self, parent: Option<Self::Value>, args: &[Self::Value]) -> Self::Funclet;
    fn cleanup_ret(
        &mut self,
        funclet: &Self::Funclet,
        unwind: Option<Self::BasicBlock>,
    ) -> Self::Value;
    fn catch_pad(&mut self, parent: Self::Value, args: &[Self::Value]) -> Self::Funclet;
    fn catch_ret(&mut self, funclet: &Self::Funclet, unwind: Self::BasicBlock) -> Self::Value;
    fn catch_switch(
        &mut self,
        parent: Option<Self::Value>,
        unwind: Option<Self::BasicBlock>,
        num_handlers: usize,
    ) -> Self::Value;
    fn add_handler(&mut self, catch_switch: Self::Value, handler: Self::BasicBlock);
    fn set_personality_fn(&mut self, personality: Self::Value);

    fn atomic_cmpxchg(
        &mut self,
        dst: Self::Value,
        cmp: Self::Value,
        src: Self::Value,
        order: AtomicOrdering,
        failure_order: AtomicOrdering,
        weak: bool,
    ) -> Self::Value;
    fn atomic_rmw(
        &mut self,
        op: AtomicRmwBinOp,
        dst: Self::Value,
        src: Self::Value,
        order: AtomicOrdering,
    ) -> Self::Value;
    fn atomic_fence(&mut self, order: AtomicOrdering, scope: SynchronizationScope);
    fn add_case(&mut self, s: Self::Value, on_val: Self::Value, dest: Self::BasicBlock);
    fn add_incoming_to_phi(&mut self, phi: Self::Value, val: Self::Value, bb: Self::BasicBlock);
    fn set_invariant_load(&mut self, load: Self::Value);

    /// Returns the ptr value that should be used for storing `val`.
    fn check_store(&mut self, val: Self::Value, ptr: Self::Value) -> Self::Value;

    /// Returns the args that should be used for a call to `llfn`.
    fn check_call<'b>(
        &mut self,
        typ: &str,
        llfn: Self::Value,
        args: &'b [Self::Value],
    ) -> Cow<'b, [Self::Value]>
    where
        [Self::Value]: ToOwned;
    fn lifetime_start(&mut self, ptr: Self::Value, size: Size);
    fn lifetime_end(&mut self, ptr: Self::Value, size: Size);

    /// If LLVM lifetime intrinsic support is enabled (i.e. optimizations
    /// on), and `ptr` is nonzero-sized, then extracts the size of `ptr`
    /// and the intrinsic for `lt` and passes them to `emit`, which is in
    /// charge of generating code to call the passed intrinsic on whatever
    /// block of generated code is targeted for the intrinsic.
    ///
    /// If LLVM lifetime intrinsic support is disabled (i.e.  optimizations
    /// off) or `ptr` is zero-sized, then no-op (does not call `emit`).
    fn call_lifetime_intrinsic(&mut self, intrinsic: &str, ptr: Self::Value, size: Size);

    fn call(
        &mut self,
        llfn: Self::Value,
        args: &[Self::Value],
        funclet: Option<&Self::Funclet>,
    ) -> Self::Value;
    fn zext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value;

    unsafe fn delete_basic_block(&mut self, bb: Self::BasicBlock);
    fn do_not_inline(&mut self, llret: Self::Value);
}
